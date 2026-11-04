use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

use crate::anchor_tracker;
use crate::controller;
use crate::controller::Controller;
use crate::l10n::L10n;
use crate::mcp_bridge::publish_map;
use crate::mcp_bridge::register_scene;
use crate::mcp_bridge::register_window;
use crate::mcp_bridge::set_game_map;
use crate::settings::AppSettings;
use crate::tangible_host::TangibleHost;
use crate::ui::art::ArtAssets;
use crate::ui::map_scene::RenderParams;
use crate::ui::map_scene::SceneLayers;
use crate::ui::map_scene::SceneState;
use crate::ui::map_scene::TwoFingerGesture;
use crate::ui::map_scene::TwoFingerStep;
use crate::ui::map_scene::{MapScene, controls};
use crate::ui::style::PxPos;
use gdk4::EventSequence;
use gdk4::EventType;
use gdk4::ModifierType;
use glib::MainContext;
use glib::Propagation;
use gtk4::EventControllerLegacy;
use gtk4::EventControllerScroll;
use gtk4::EventControllerScrollFlags;
use gtk4::EventSequenceState;
use gtk4::GestureClick;
use libadwaita as adw;
use libadwaita::prelude::*;
use qdo36_map::GameMap;
use qdo36_mcp::MapInfo;
use qdo36_tangibles::AnchorConfig;
use qdo36_tangibles::AnchorPattern;
use qdo36_tangibles::AnchorPose;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::DaemonError;
use qdo36_tangibles::GraphQlSource;
use qdo36_tangibles::MapScale;
use qdo36_tangibles::SurfaceMm;
use qdo36_tangibles::TangibleEvent;
use tangible_objects_model::RotationRad;
use tangible_objects_model::Vec2;
use tracing::info;
use tracing::warn;

/// Orbit radius below which Ctrl+right-drag stays inert (widget px).
const MOUSE_ROTATE_MIN_RADIUS_PX: f64 = 16.0;
/// Zoom factor per mouse-wheel notch (matches the zoom buttons).
const WHEEL_ZOOM_STEP: f64 = 1.25;

/// The application window: a fullscreen stack switching between the
/// daemon setup screen and the map scene.
pub struct AppWindow {
    /// The GTK window.
    pub window: adw::ApplicationWindow,
    /// The screen stack (setup ↔ map).
    stack: gtk4::Stack,
    /// The setup status page.
    setup: adw::StatusPage,
    /// The map scene widget.
    scene: MapScene,
    /// Fluent lookups.
    l10n: Rc<L10n>,
    /// Persisted settings snapshot.
    settings: AppSettings,
}

impl AppWindow {
    /// Builds the window and kicks off the daemon handshake.
    pub fn new(app: &adw::Application, settings: AppSettings, l10n: Rc<L10n>) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder().application(app).build();

        let stack = gtk4::Stack::new();
        let setup = adw::StatusPage::builder()
            .title(l10n.msg("app-title"))
            .description(l10n.msg("daemon-connecting"))
            .icon_name("qdo-map-symbolic")
            .build();
        stack.add_named(&setup, Some("setup"));

        let scene = MapScene::new();
        // The map page carries the on-screen control bar as an
        // overlay at the screen edge (independent of the map camera).
        let map_page = gtk4::Overlay::new();
        map_page.set_child(Some(&scene));
        map_page.add_overlay(&controls());
        stack.add_named(&map_page, Some("map"));

        window.set_content(Some(&stack));
        register_window(&window);

        let this = Rc::new(Self {
            window,
            stack,
            setup,
            scene,
            l10n,
            settings,
        });
        this.start_handshake();
        this
    }

    /// Runs the blocking daemon handshake on a worker thread, then
    /// hands results back to the glib main loop.
    fn start_handshake(self: &Rc<Self>) {
        let Ok(controller) = controller::init_controller() else {
            self.setup.set_description(Some("internal: tokio runtime failed"));
            return;
        };

        // The worker thread drives the blocking handshake via the
        // runtime Handle (Send); the controller itself is thread-local
        // and cannot be reached from there.
        let handle = controller.borrow().host().handle();
        let endpoint = self.settings.daemon_url.clone();
        let city = self.settings.city;
        let (tx, rx) = async_channel::bounded(1);
        std::thread::spawn(move || {
            let result = AnchorPattern::for_city(city).and_then(|pattern| TangibleHost::connect_and_subscribe(&handle, &endpoint, &pattern));
            let _ = tx.send_blocking(result);
        });

        let map = GameMap::for_city(city).map(Rc::new).ok();
        let scale = self.settings.map_scale;
        let this = self.clone();
        MainContext::default().spawn_local(async move {
            match rx.recv().await {
                Ok(Ok((source, _pattern_id, events))) => {
                    this.on_connected(&controller, source, events, map, scale);
                }
                Ok(Err(err)) => {
                    warn!("daemon handshake failed: {err}");
                    let status = match &err {
                        DaemonError::Connect { .. } => this.l10n.msg("daemon-unreachable"),
                        _ => err.to_string(),
                    };
                    this.setup.set_description(Some(&status));
                }
                Err(_) => this.setup.set_description(Some("connect channel closed")),
            }
        });
    }

    /// Connect succeeded: store the source, report the surface size in
    /// the status line and start the tangible event pump feeding the
    /// scene.
    fn on_connected(
        self: &Rc<Self>,
        controller: &Rc<RefCell<Controller>>,
        source: GraphQlSource,
        events: async_channel::Receiver<TangibleEvent>,
        map: Option<Rc<GameMap>>,
        scale: MapScale,
    ) {
        let (status, surface) = {
            let mut controller = controller.borrow_mut();
            let surface = controller.host().surface(&source).ok();
            controller.set_source(source);
            let status = match surface {
                Some(surface) => format!("{} ({:.0} x {:.0} mm)", self.l10n.msg("daemon-ready"), surface.width.as_f64(), surface.height.as_f64()),
                None => self.l10n.msg("daemon-ready"),
            };
            (status, surface)
        };
        let prompt = self.l10n.msg("place-tower");
        self.setup.set_description(Some(&format!("{status}\n\n{prompt}")));
        info!("daemon handshake complete");

        let Some(map) = map else {
            self.setup.set_description(Some("map dataset failed to load"));
            return;
        };
        let state = Rc::new(RefCell::new(self.scene_state(&map, surface, scale)));
        let stack = self.stack.clone();
        register_scene(&state, &self.scene, Rc::new(move || stack.set_visible_child_name("map")));

        // Tap hit-testing: widget-px -> map-mm -> nearest station
        // within the snap radius; the selection rings it.
        let gesture = GestureClick::new();
        {
            let state = state.clone();
            let scene = self.scene.clone();
            gesture.connect_released(move |_, _, px, py| {
                let station = scene.station_at(&state.borrow(), Vec2::new(px, py));
                if let Some(station) = station {
                    state.borrow_mut().selected = Some(station);
                    scene.update(Rc::new(state.borrow().clone()));
                }
            });
        }

        // Two-finger pan/pinch/rotate: while no tower sits on the
        // table (virtual anchor) the session locks into move or
        // rotate on the first threshold crossing - move shifts the
        // view pan and pinch-zooms, rotate turns the virtual anchor
        // pose. A tracked tower moves the map itself, so the
        // controller stays inert then. Once a second finger lands
        // with a virtual anchor, the tap click is denied - the
        // lifting fingers must not select a station. Registered
        // before the click so it always sees the raw touch stream.
        let touches = Rc::new(RefCell::new(HashMap::<EventSequence, PxPos>::new()));
        let two_finger = Rc::new(RefCell::new(TwoFingerGesture::new()));
        // Mouse view control for desktop sessions, same
        // virtual-anchor gate: Ctrl+left drag pans the map point
        // under the press point, Ctrl+right drag orbits the map
        // around the press point.
        let mouse_pan: Rc<Cell<Option<PxPos>>> = Rc::new(Cell::new(None));
        let mouse_rotate_pivot: Rc<Cell<Option<PxPos>>> = Rc::new(Cell::new(None));
        let mouse_rotate_angle: Rc<Cell<Option<f64>>> = Rc::new(Cell::new(None));
        let legacy = EventControllerLegacy::new();
        {
            let state = state.clone();
            let scene = self.scene.clone();
            let gesture = gesture.clone();
            legacy.connect_event(move |_, event| {
                let repair_pair = |touches: &HashMap<EventSequence, PxPos>| {
                    let pair = if touches.len() == 2 {
                        let mut iter = touches.values();
                        iter.next().zip(iter.next()).map(|(first, second)| (*first, *second))
                    } else {
                        None
                    };
                    match pair {
                        Some((first, second)) => two_finger.borrow_mut().pair_started(first, second),
                        None => two_finger.borrow_mut().pair_broken(),
                    }
                };
                match event.event_type() {
                    EventType::TouchBegin => {
                        if let Some((x, y)) = event.position() {
                            touches.borrow_mut().insert(event.event_sequence(), Vec2::new(x, y));
                        }
                        repair_pair(&touches.borrow());
                        if touches.borrow().len() == 2 && matches!(state.borrow().anchor, AnchorState::Virtual(_)) {
                            gesture.set_state(EventSequenceState::Denied);
                        }
                    }
                    EventType::TouchUpdate => {
                        let Some((x, y)) = event.position() else {
                            return Propagation::Proceed;
                        };
                        let sequence = event.event_sequence();
                        let position = Vec2::new(x, y);
                        let (previous, other) = {
                            let mut touches = touches.borrow_mut();
                            let previous = touches.insert(sequence.clone(), position);
                            let other = match (previous, touches.len()) {
                                (Some(_), 2) => touches.iter().find(|(key, _)| **key != sequence).map(|(_, pos)| *pos),
                                _ => None,
                            };
                            (previous, other)
                        };
                        let (Some(previous), Some(other)) = (previous, other) else {
                            return Propagation::Proceed;
                        };
                        let Some(step) = two_finger.borrow_mut().update(previous, position, other) else {
                            return Propagation::Proceed;
                        };
                        {
                            let mut state = state.borrow_mut();
                            if !matches!(state.anchor, AnchorState::Virtual(_)) {
                                return Propagation::Proceed;
                            }
                            match step {
                                TwoFingerStep::Move {
                                    centroid_before,
                                    centroid_after,
                                    span_factor,
                                } => {
                                    let zoom = (state.scale.factor() * span_factor.get()).clamp(MapScale::MIN.factor(), MapScale::MAX.factor());
                                    let Ok(scale) = MapScale::try_new(zoom) else {
                                        return Propagation::Proceed;
                                    };
                                    let pose = state.anchor.pose();
                                    state.pan = scene.anchored_pan(&state, centroid_before, centroid_after, pose, scale);
                                    state.scale = scale;
                                }
                                TwoFingerStep::Rotate { pivot, delta_rad } => {
                                    let pose = state.anchor.pose();
                                    let pose = AnchorPose {
                                        rotation_rad: pose.rotation_rad + delta_rad,
                                        ..pose
                                    };
                                    state.pan = scene.anchored_pan(&state, pivot, pivot, pose, state.scale);
                                    state.anchor = AnchorState::Virtual(pose);
                                }
                            }
                        }
                        gesture.set_state(EventSequenceState::Denied);
                        scene.update(Rc::new(state.borrow().clone()));
                    }
                    EventType::TouchEnd | EventType::TouchCancel => {
                        touches.borrow_mut().remove(&event.event_sequence());
                        repair_pair(&touches.borrow());
                    }
                    EventType::ButtonPress => {
                        let Some((x, y)) = event.position() else {
                            return Propagation::Proceed;
                        };
                        let armed = event.modifier_state().contains(ModifierType::CONTROL_MASK) && matches!(state.borrow().anchor, AnchorState::Virtual(_));
                        if !armed {
                            return Propagation::Proceed;
                        }
                        let position = Vec2::new(x, y);
                        match event.downcast_ref::<gdk4::ButtonEvent>().map(|button| button.button()) {
                            Some(gdk4::BUTTON_PRIMARY) => {
                                mouse_pan.set(Some(position));
                                gesture.set_state(EventSequenceState::Denied);
                            }
                            Some(gdk4::BUTTON_SECONDARY) => {
                                mouse_rotate_pivot.set(Some(position));
                                mouse_rotate_angle.set(None);
                                gesture.set_state(EventSequenceState::Denied);
                            }
                            _ => {}
                        }
                    }
                    EventType::ButtonRelease => match event.downcast_ref::<gdk4::ButtonEvent>().map(|button| button.button()) {
                        Some(gdk4::BUTTON_PRIMARY) => mouse_pan.set(None),
                        Some(gdk4::BUTTON_SECONDARY) => {
                            mouse_rotate_pivot.set(None);
                            mouse_rotate_angle.set(None);
                        }
                        _ => {}
                    },
                    EventType::MotionNotify => {
                        let Some((x, y)) = event.position() else {
                            return Propagation::Proceed;
                        };
                        let position = Vec2::new(x, y);
                        // Incremental pan: pin the point under the
                        // previous cursor position onto the new one (a
                        // fixed press `from` would re-pin an
                        // already-shifted point and compound the pan).
                        let previous_cursor = mouse_pan.get();
                        if previous_cursor.is_some() {
                            mouse_pan.set(Some(position));
                        }
                        let applied = {
                            let mut state = state.borrow_mut();
                            if !matches!(state.anchor, AnchorState::Virtual(_)) {
                                false
                            } else if let Some(previous) = previous_cursor {
                                let pose = state.anchor.pose();
                                state.pan = scene.anchored_pan(&state, previous, position, pose, state.scale);
                                true
                            } else if let Some(pivot) = mouse_rotate_pivot.get() {
                                let radius = (position.x - pivot.x).hypot(position.y - pivot.y);
                                if radius < MOUSE_ROTATE_MIN_RADIUS_PX {
                                    false
                                } else {
                                    let angle = (position.y - pivot.y).atan2(position.x - pivot.x);
                                    match mouse_rotate_angle.replace(Some(angle)) {
                                        Some(previous) => {
                                            let pose = state.anchor.pose();
                                            let pose = AnchorPose {
                                                rotation_rad: pose.rotation_rad + RotationRad::new(wrap_pi(angle - previous)),
                                                ..pose
                                            };
                                            state.pan = scene.anchored_pan(&state, pivot, pivot, pose, state.scale);
                                            state.anchor = AnchorState::Virtual(pose);
                                            true
                                        }
                                        None => false,
                                    }
                                }
                            } else {
                                false
                            }
                        };
                        if applied {
                            scene.update(Rc::new(state.borrow().clone()));
                        }
                    }
                    _ => {}
                }
                Propagation::Proceed
            });
        }
        self.scene.add_controller(legacy);
        self.scene.add_controller(gesture.clone());

        let scroll = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        {
            let state = state.clone();
            let scene = self.scene.clone();
            scroll.connect_scroll(move |scroll, _dx, dy| {
                if !scroll.current_event_state().contains(ModifierType::CONTROL_MASK) {
                    return Propagation::Proceed;
                }
                {
                    let mut state = state.borrow_mut();
                    if !matches!(state.anchor, AnchorState::Virtual(_)) {
                        return Propagation::Proceed;
                    }
                    let zoom = (state.scale.factor() * WHEEL_ZOOM_STEP.powf(-dy)).clamp(MapScale::MIN.factor(), MapScale::MAX.factor());
                    let Ok(scale) = MapScale::try_new(zoom) else {
                        return Propagation::Proceed;
                    };
                    if let Some((x, y)) = scroll.current_event().and_then(|event| event.position()) {
                        let pose = state.anchor.pose();
                        state.pan = scene.anchored_pan(&state, Vec2::new(x, y), Vec2::new(x, y), pose, scale);
                    }
                    state.scale = scale;
                }
                scene.update(Rc::new(state.borrow().clone()));
                Propagation::Stop
            });
            self.scene.add_controller(scroll);
        }

        // Event pump: tangible events arrive on the glib loop; every
        // pose updates the scene; the first tracked anchor advances
        // to the map screen.
        let this = self.clone();
        MainContext::default().spawn_local(async move {
            while let Ok(event) = events.recv().await {
                controller::with_controller(|controller| {
                    let tracked = matches!(event, TangibleEvent::Added(_) | TangibleEvent::Moved(_));
                    let loss_started = {
                        let mut controller = controller.borrow_mut();
                        let loss_started = controller.apply_event(&event);
                        let anchor = controller.anchor();
                        let mut state = state.borrow_mut();
                        // A re-anchored tower carries the map again:
                        // drop the finger pan so the rendered anchor
                        // lands back under the tangible.
                        if tracked && matches!(state.anchor, AnchorState::Virtual(_)) {
                            state.pan = Vec2::new(0.0, 0.0);
                        }
                        state.anchor = anchor;
                        crate::mcp_bridge::publish_state(&anchor, surface.as_ref());
                        loss_started
                    };
                    this.scene.update(Rc::new(state.borrow().clone()));
                    if tracked && this.stack.visible_child_name().is_some_and(|name| name == "setup") {
                        this.stack.set_visible_child_name("map");
                    }
                    if loss_started {
                        // The anchor was lost: re-check once the grace
                        // period elapsed; a re-tracked tower meanwhile
                        // cancels the downgrade.
                        let state = state.clone();
                        let scene = this.scene.clone();
                        MainContext::default().spawn_local(async move {
                            glib::timeout_future(anchor_tracker::ANCHOR_LOSS_GRACE).await;
                            controller::with_controller(|controller| {
                                let mut controller = controller.borrow_mut();
                                if controller.settle_anchor_loss() {
                                    state.borrow_mut().anchor = controller.anchor();
                                    scene.update(Rc::new(state.borrow().clone()));
                                }
                            });
                        });
                    }
                });
            }
        });
    }

    /// Builds the scene state: map, anchor config (from the city's
    /// anchor spec), virtual anchor and surface.
    fn scene_state(&self, map: &Rc<GameMap>, surface: Option<SurfaceMm>, scale: MapScale) -> SceneState {
        let anchor_pos = map
            .meta()
            .anchor
            .as_ref()
            .and_then(|spec| map.station(spec.station))
            .or_else(|| map.stations().next())
            .map(|station| station.pos)
            .unwrap_or_else(|| Vec2::new(0.0, 0.0));
        publish_map(MapInfo::from_map(map, Some(anchor_pos)));
        set_game_map(map.clone());
        let mut config = AnchorConfig::at_station(map.bounds(), anchor_pos);
        config.heading_offset = self.settings.heading_offset_deg.to_rotation_rad();
        SceneState {
            map: map.clone(),
            config,
            // The event pump starts before any tangible event - the
            // virtual fallback pose.
            anchor: AnchorState::default_virtual(),
            surface,
            scale,
            selected: None,
            display_rotation: self.settings.display_rotation,
            layers: SceneLayers::default(),
            pan: Vec2::new(0.0, 0.0),
            params: RenderParams::default(),
            // Inside the bounds' top-right corner - the previous
            // margin position (x_max + 16, y_min - 16) clipped away.
            compass_pos: Vec2::new(map.bounds().x_max.as_f64() - 24.0, map.bounds().y_min.as_f64() + 24.0),
            style: self.settings.map_style,
            assets: ArtAssets::detect(),
            anchor_hint: Some(self.l10n.msg("tower-missing")),
        }
    }
}

/// Wraps an angle delta into [-pi, pi] (the orbit ray is directed,
/// unlike the mod-pi finger line).
fn wrap_pi(delta: f64) -> f64 {
    (delta + PI).rem_euclid(2.0 * PI) - PI
}
