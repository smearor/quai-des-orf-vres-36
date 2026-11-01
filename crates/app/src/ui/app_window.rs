use std::cell::RefCell;
use std::rc::Rc;

use glib::MainContext;
use libadwaita as adw;
use libadwaita::prelude::*;
use qdo36_map::GameMap;
use qdo36_tangibles::AnchorConfig;
use qdo36_tangibles::AnchorPattern;
use qdo36_tangibles::DaemonError;
use qdo36_tangibles::GraphQlSource;
use qdo36_tangibles::MapScale;
use qdo36_tangibles::SurfaceMm;
use qdo36_tangibles::TangibleEvent;
use tangible_objects_model::Vec2;
use tracing::info;
use tracing::warn;

use crate::controller;
use crate::controller::Controller;
use crate::l10n::L10n;
use crate::settings::AppSettings;
use crate::tangible_host::TangibleHost;
use crate::ui::map_scene::MapScene;
use crate::ui::map_scene::SceneState;

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
        stack.add_named(&scene, Some("map"));

        window.set_content(Some(&stack));

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
        self.setup.set_description(Some(&status));
        info!("daemon handshake complete");

        let Some(map) = map else {
            self.setup.set_description(Some("map dataset failed to load"));
            return;
        };
        let state = Rc::new(RefCell::new(self.scene_state(&map, surface, scale)));

        // Event pump: tangible events arrive on the glib loop; every
        // pose updates the scene; the first tracked anchor advances
        // to the map screen.
        let this = self.clone();
        MainContext::default().spawn_local(async move {
            while let Ok(event) = events.recv().await {
                controller::with_controller(|controller| {
                    let tracked = matches!(event, TangibleEvent::Added(_) | TangibleEvent::Moved(_));
                    {
                        let mut controller = controller.borrow_mut();
                        controller.apply_event(&event);
                        state.borrow_mut().anchor = controller.anchor();
                    }
                    this.scene.update(Rc::new(state.borrow().clone()));
                    if tracked && this.stack.visible_child_name().is_some_and(|name| name == "setup") {
                        this.stack.set_visible_child_name("map");
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
        let config = AnchorConfig::at_station(map.bounds(), anchor_pos);
        SceneState {
            map: map.clone(),
            config,
            // The event pump starts before any tangible event - the
            // virtual fallback pose.
            anchor: qdo36_tangibles::AnchorState::default_virtual(),
            surface,
            scale,
            selected: None,
            display_rotation: self.settings.display_rotation,
        }
    }
}
