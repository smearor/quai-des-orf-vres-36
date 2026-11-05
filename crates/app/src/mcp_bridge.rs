//! MCP bridge: publishes the app state snapshot the `qdo36-mcp`
//! endpoint serves, and drains the command channel into the
//! app-owned [`GameHost`].
//!
//! The GTK side *writes* a typed document into a `watch` channel and
//! *executes* `GameRequest`s on the main loop; the MCP thread reads
//! the document and forwards commands - no GTK access from the
//! server, ever (WP05 R5).

use crate::ui::map_scene::LabelMode;
use crate::ui::map_scene::MapScene;
use crate::ui::map_scene::RenderStage;
use crate::ui::map_scene::SceneLayers;
use crate::ui::map_scene::SceneState;
use crate::ui::style::MapStyle;
use crate::ui::style::PxPos;
use async_channel::unbounded;
use glib::MainContext;
use libadwaita as adw;
use qdo36_host::GameHost;
use qdo36_map::LineName;
use qdo36_map::StationId;
use qdo36_map::Transport;
use qdo36_map::{City, GameMap};
use qdo36_mcp::CommandKind;
use qdo36_mcp::CommandReply;
use qdo36_mcp::CommandResult;
use qdo36_mcp::GameRequest;
use qdo36_mcp::LayerMask;
use qdo36_mcp::MapInfo;
use qdo36_mcp::McpError;
use qdo36_mcp::ScreenshotReply;
use qdo36_mcp::StateSnapshot;
use qdo36_mcp::StyleName;
use qdo36_mcp::spawn_http;
use qdo36_mcp::{AnchorStateSnapshot, EntityKind, SceneEntity};
use qdo36_session::GameSession;
use qdo36_tangibles::MapScale;
use qdo36_tangibles::ScaleFactor;
use qdo36_tangibles::SurfaceMm;
use qdo36_tangibles::{AnchorPose, AnchorState};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::RotationRad;
use tangible_objects_model::Vec2;
use tokio::sync::watch::Sender;
use tokio::sync::watch::channel;
use tracing::info;

thread_local! {
    /// Latest-state channel feeding the MCP `qdo36://state` resource
    /// and its subscribers; installed by [`attach`].
    static MCP_STATE_WATCH: RefCell<Option<Sender<StateSnapshot>>> = const { RefCell::new(None) };
    /// The app-owned game session answering MCP commands; installed
    /// by [`attach`] (no host without `--mcp-listen`).
    static GAME_HOST: RefCell<Option<GameHost>> = const { RefCell::new(None) };
    /// The app window the `screenshot` command renders; installed by
    /// [`register_window`].
    static APP_WINDOW: RefCell<Option<adw::ApplicationWindow>> = const { RefCell::new(None) };
    /// The live scene (state + widget + show-map callback) the view
    /// commands mutate and `scene_layout` enumerates; installed by
    /// [`register_scene`].
    static SCENE: RefCell<Option<SceneHandle>> = const { RefCell::new(None) };
}

/// The registered scene: shared state, the scene widget and a
/// callback lifting the map page over the setup page.
type SceneHandle = (Rc<RefCell<SceneState>>, MapScene, Rc<dyn Fn()>);

/// Runs `f` against the registered scene pair.
fn with_scene<T>(f: impl FnOnce(&Rc<RefCell<SceneState>>, &MapScene) -> T) -> Option<T> {
    SCENE.with(|slot| slot.borrow().as_ref().map(|(state, scene, _)| f(state, scene)))
}

/// Runs `f` against the game host; the host exists only while the MCP
/// endpoint is attached.
fn with_game_host<T>(f: impl FnOnce(&mut GameHost) -> T) -> Option<T> {
    GAME_HOST.with(|slot| slot.borrow_mut().as_mut().map(f))
}

/// Starts the MCP Streamable-HTTP endpoint on `addr`, installs the
/// state channel and drains `GameRequest`s on the GTK main loop.
/// No-op friendly: the app is unchanged when `--mcp-listen` is not
/// passed.
pub fn attach(addr: SocketAddr) {
    let (watch_tx, watch_rx) = channel(StateSnapshot::default());
    let (command_tx, command_rx) = unbounded::<GameRequest>();
    MCP_STATE_WATCH.with(|slot| *slot.borrow_mut() = Some(watch_tx));
    GAME_HOST.with(|slot| *slot.borrow_mut() = Some(GameHost::with_rng(StdRng::from_rng(&mut rand::rng()))));
    spawn_http(command_tx, watch_rx, addr);
    info!(%addr, "MCP endpoint attached");
    MainContext::default().spawn_local(async move {
        while let Ok(request) = command_rx.recv().await {
            // View/scene commands are app concerns, not session ones -
            // the game host never sees them.
            let reply = match &request.command {
                CommandKind::Screenshot => mcp_screenshot(),
                CommandKind::SceneLayout => scene_layout(),
                CommandKind::SetView { zoom, rotate, pan, compass } => set_view(*zoom, *rotate, *pan, *compass),
                CommandKind::Goto { stations, zoom } => goto(stations, *zoom),
                CommandKind::SetLayers { mask } => set_layers(mask),
                CommandKind::SetStyle { style } => set_style(*style),
                CommandKind::SetAnchor { center, rotation_rad } => set_anchor(*center, *rotation_rad),
                CommandKind::ReloadMap => reload_map(),
                CommandKind::SetParams {
                    station_scale,
                    landmark_scale,
                    metro_scale,
                    ring_width,
                    label_scale,
                } => set_params(*station_scale, *landmark_scale, *metro_scale, *ring_width, *label_scale),
                _ => with_game_host(|host| host.handle(&request.command)).unwrap_or(Err(McpError::ChannelClosed)),
            };
            // Publish after every command - mutations become visible
            // to move_log/game_config/game_state subscribers.
            with_game_host(|host| publish_game(host.session_cloned()));
            let _ = request.reply.send(reply);
        }
    });
}

/// Registers the app window the `screenshot` command renders.
/// Called once when the window is built.
pub fn register_window(window: &adw::ApplicationWindow) {
    APP_WINDOW.with(|slot| *slot.borrow_mut() = Some(window.clone()));
}

/// Renders the registered window into a PNG texture; `None` when no
/// window or surface is available.
fn screenshot_texture() -> Option<gdk4::Texture> {
    use gtk4::prelude::*;

    let window = APP_WINDOW.with(|slot| slot.borrow().clone())?;
    let surface = window.surface()?;
    let snapshot = gtk4::Snapshot::new();
    gtk4::WidgetPaintable::new(Some(&window)).snapshot(&snapshot, f64::from(window.width()), f64::from(window.height()));
    let node = snapshot.to_node()?;
    let renderer = gtk4::gsk::Renderer::for_surface(&surface)?;
    let viewport = gtk4::graphene::Rect::new(0.0, 0.0, window.width() as f32, window.height() as f32);
    let texture = renderer.render_texture(&node, Some(&viewport));
    // A renderer may only be disposed while unrealized - dropping it
    // while realized on the surface asserts in GSK.
    renderer.unrealize();
    Some(texture)
}

/// The `screenshot` command: PNG of the window, base64-encoded.
fn mcp_screenshot() -> CommandReply {
    use gdk4::prelude::TextureExt;

    let Some(texture) = screenshot_texture() else {
        return Err(McpError::StateUnavailable);
    };
    let bytes = texture.save_to_png_bytes();
    Ok(CommandResult::Screenshot(ScreenshotReply {
        png_base64: glib::base64_encode(bytes.as_ref()).to_string(),
        width: Pixels::new(f64::from(texture.width())),
        height: Pixels::new(f64::from(texture.height())),
    }))
}

/// Registers the live scene for `scene_layout`/`set_view`/
/// `set_layers`. Called once when the map scene is built.
pub fn register_scene(state: &Rc<RefCell<SceneState>>, scene: &MapScene, show_map: Rc<dyn Fn()>) {
    SCENE.with(|slot| *slot.borrow_mut() = Some((state.clone(), scene.clone(), show_map)));
}

/// The `set_anchor` command: virtual anchor pose on the surface +
/// lift the map page over the setup page.
fn set_anchor(center: Vec2<Millimeters>, rotation_rad: Option<RotationRad>) -> CommandReply {
    let Some((state, scene, show_map)) = SCENE.with(|slot| slot.borrow().clone()) else {
        return Err(McpError::ChannelClosed);
    };
    {
        let mut state = state.borrow_mut();
        state.anchor = AnchorState::Virtual(AnchorPose {
            center,
            rotation_rad: rotation_rad.unwrap_or(RotationRad::new(0.0)),
        });
        scene.update(Rc::new(state.clone()));
    }
    show_map();
    Ok(CommandResult::AnchorSet)
}

/// The `set_params` command: global render factors on the live scene.
fn set_params(
    station_scale: Option<ScaleFactor>,
    landmark_scale: Option<ScaleFactor>,
    metro_scale: Option<ScaleFactor>,
    ring_width: Option<ScaleFactor>,
    label_scale: Option<ScaleFactor>,
) -> CommandReply {
    with_scene(|state, scene| {
        let mut state = state.borrow_mut();
        let p = &mut state.params;
        if let Some(v) = station_scale {
            p.station_scale = v;
        }
        if let Some(v) = landmark_scale {
            p.landmark_scale = v;
        }
        if let Some(v) = metro_scale {
            p.metro_scale = v;
        }
        if let Some(v) = ring_width {
            p.ring_width = v;
        }
        if let Some(v) = label_scale {
            p.label_scale = v;
        }
        scene.update(Rc::new(state.clone()));
        Ok(CommandResult::ParamsApplied)
    })
    .unwrap_or(Err(McpError::StateUnavailable))
}

/// The `reload_map` command: re-reads the city dataset from disk
/// (dev checkout) or the bundled copy, swaps the scene map and
/// hands it to the game host.
fn reload_map() -> CommandReply {
    with_scene(|state, scene| {
        let map = {
            let state = state.borrow();
            let city: City = state
                .map
                .meta()
                .name
                .as_str()
                .parse()
                .map_err(|e| McpError::InvalidParams(format!("unknown city: {e}")))?;
            let text = city
                .dataset_path()
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .unwrap_or_else(|| city.dataset().to_string());
            Rc::new(
                text.parse::<GameMap>()
                    .map_err(|e| McpError::InvalidParams(format!("dataset reload failed: {e}")))?,
            )
        };
        set_game_map(map.clone());
        publish_map(MapInfo::from_map(&map, Some(state.borrow().config.anchor_map_pos)));
        state.borrow_mut().map = map;
        scene.update(Rc::new(state.borrow().clone()));
        Ok(CommandResult::MapReloaded)
    })
    .unwrap_or(Err(McpError::StateUnavailable))
}

/// The `scene_layout` command: every station and landmark with
/// map-mm and widget-px positions under the current transform.
fn scene_layout() -> CommandReply {
    use gtk4::prelude::WidgetExt;

    let Some((entities,)) = with_scene(|state, scene| {
        let state = state.borrow();
        // The scene fills the window; fall back to its size when the
        // widget reports no allocation yet.
        let (width, height) = match (scene.width(), scene.height()) {
            (w, h) if w > 0 && h > 0 => (w, h),
            _ => APP_WINDOW
                .with(|slot| slot.borrow().as_ref().map(|w| (w.width(), w.height())))
                .unwrap_or((0, 0)),
        };
        let stage = RenderStage::new(&state, state.anchor.pose(), Pixels::new(width as f64), Pixels::new(height as f64));
        let to_px = |pos| stage.to_px(pos);
        let stations = state.map.stations().map(|s| SceneEntity {
            kind: EntityKind::Station,
            name: s.name.to_string(),
            station: Some(s.id),
            mm: s.pos,
            px: to_px(s.pos),
        });
        let landmarks = state.map.backdrop().landmarks.iter().map(|l| SceneEntity {
            kind: EntityKind::Landmark,
            name: l.name.to_string(),
            station: None,
            mm: l.pos,
            px: to_px(l.pos),
        });
        (stations.chain(landmarks).collect::<Vec<_>>(),)
    }) else {
        return Err(McpError::StateUnavailable);
    };
    Ok(CommandResult::Layout(qdo36_mcp::SceneLayout { entities }))
}

/// The `set_view` command: pan/rotate/zoom/compass on the live scene.
fn set_view(zoom: Option<MapScale>, rotate: Option<DisplayRotation>, pan: Option<Vec2<Millimeters>>, compass: Option<Vec2<Millimeters>>) -> CommandReply {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            if let Some(scale) = zoom {
                state.scale = scale;
            }
            if let Some(rotation) = rotate {
                state.display_rotation = rotation;
            }
            if let Some(offset) = pan {
                state.pan = offset;
            }
            if let Some(pos) = compass {
                state.compass_pos = pos;
            }
        }
        scene.update(Rc::new(state.borrow().clone()));
        Ok(CommandResult::ViewApplied)
    })
    .unwrap_or(Err(McpError::StateUnavailable))
}

/// On-screen control helpers - the same scene mutation path the
/// MCP commands use, callable from GTK callbacks (main thread).
/// Multiplies the view zoom by `factor` (zoom buttons: 1.25 / 0.8).
pub(crate) fn zoom_by(factor: f64) {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            let zoom = (state.scale.factor() * factor).clamp(MapScale::MIN.factor(), MapScale::MAX.factor());
            if let Ok(scale) = MapScale::try_new(zoom) {
                state.scale = scale;
            }
        }
        scene.update(Rc::new(state.borrow().clone()));
    });
}

/// Resets zoom to `MapScale::ONE` and clears the pan.
/// The `goto_*` commands: centers the view on the stations'
/// midpoint. Two stations auto-zoom so the hop fills roughly 55%
/// of the window's smaller side; one station keeps the zoom unless
/// `zoom` overrides.
fn goto(stations: &[StationId], zoom: Option<MapScale>) -> CommandReply {
    use gtk4::prelude::WidgetExt;

    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            let positions: Vec<Vec2<Millimeters>> = stations.iter().filter_map(|id| state.map.station(*id)).map(|s| s.pos).collect();
            if positions.is_empty() {
                return Err(McpError::InvalidParams(format!("unknown station(s) {stations:?}")));
            }
            let (width, height) = match (scene.width(), scene.height()) {
                (w, h) if w > 0 && h > 0 => (w, h),
                _ => APP_WINDOW
                    .with(|slot| slot.borrow().as_ref().map(|w| (w.width(), w.height())))
                    .unwrap_or((0, 0)),
            };
            let stage = RenderStage::new(&state, state.anchor.pose(), Pixels::new(width as f64), Pixels::new(height as f64));
            let center = Vec2::new(
                positions.iter().map(|pos| pos.x).sum::<f64>() / positions.len() as f64,
                positions.iter().map(|pos| pos.y).sum::<f64>() / positions.len() as f64,
            );
            state.pan = stage.pan_for(center, PxPos::new(width as f64 / 2.0, height as f64 / 2.0));
            // Auto-fit: widget px per map-mm is zoom * fit.scale, so
            // the zoom landing the station distance on 55% of the
            // smaller side is target_px / (distance * fit.scale).
            let scale = zoom.or_else(|| {
                let [a, b] = positions.as_slice() else { return None };
                let distance = (b.x - a.x).hypot(b.y - a.y);
                let fit = 0.55 * (width.min(height) as f64) / (distance * stage.fit.scale.as_f64());
                MapScale::try_new(fit.clamp(MapScale::MIN.factor(), MapScale::MAX.factor())).ok()
            });
            if let Some(scale) = scale {
                state.scale = scale;
            }
        }
        scene.update(Rc::new(state.borrow().clone()));
        Ok(CommandResult::ViewApplied)
    })
    .unwrap_or(Err(McpError::StateUnavailable))
}

pub(crate) fn reset_view() {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            state.scale = MapScale::ONE;
            state.pan = Vec2::new(0.0, 0.0);
        }
        scene.update(Rc::new(state.borrow().clone()));
    });
}

/// Toggles the label orientation (radial <-> screen).
pub(crate) fn toggle_label_mode() {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            state.layers.label_mode = match state.layers.label_mode {
                LabelMode::Radial => LabelMode::Screen,
                LabelMode::Screen => LabelMode::Radial,
            };
        }
        scene.update(Rc::new(state.borrow().clone()));
    });
}

/// Toggles the landmark orientation (screen <-> radial).
pub(crate) fn toggle_landmark_mode() {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            state.layers.landmark_mode = match state.layers.landmark_mode {
                LabelMode::Radial => LabelMode::Screen,
                LabelMode::Screen => LabelMode::Radial,
            };
        }
        scene.update(Rc::new(state.borrow().clone()));
    });
}

/// Mutates the layer set (overlay menu checkboxes).
pub(crate) fn mutate_layers(f: impl FnOnce(&mut SceneLayers) + 'static) {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            f(&mut state.layers);
        }
        scene.update(Rc::new(state.borrow().clone()));
    });
}

/// The current layer set (checkbox initial states on the UI).
pub(crate) fn current_layers() -> Option<SceneLayers> {
    with_scene(|state, _| state.borrow().layers.clone())
}

/// Mutates the artwork style (style-menu buttons; also the
/// `set_style` MCP command).
pub(crate) fn set_map_style(style: MapStyle) {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            state.style = style;
        }
        scene.update(Rc::new(state.borrow().clone()));
    });
}

/// The current artwork style (style-menu initial state on the UI).
pub(crate) fn current_style() -> Option<MapStyle> {
    with_scene(|state, _| state.borrow().style)
}

/// The `set_style` command: switch the map artwork at runtime.
fn set_style(style: StyleName) -> CommandReply {
    with_scene(|_, _| set_map_style(style.into()))
        .map(|_| CommandResult::StyleApplied)
        .ok_or(McpError::StateUnavailable)
}

/// The `set_layers` command: layer-visibility toggles.
fn set_layers(mask: &LayerMask) -> CommandReply {
    with_scene(|state, scene| {
        {
            let mut state = state.borrow_mut();
            let l = &mut state.layers;
            if let Some(v) = mask.water {
                l.water = v;
            }
            if let Some(v) = mask.landmarks {
                l.landmarks = v;
            }
            let t = &mut l.transports;
            if let Some(v) = mask.taxi {
                t.taxi = v;
            }
            if let Some(v) = mask.bus {
                t.bus = v;
            }
            if let Some(v) = mask.metro {
                t.metro = v;
            }
            if let Some(v) = mask.batobus {
                t.batobus = v;
            }
            if let Some(v) = mask.underground {
                t.underground = v;
            }
            if let Some(v) = mask.parallel_transports {
                l.parallel_transports = v;
            }
            if let Some(lines) = &mask.hide_lines {
                l.hidden_lines = lines.iter().map(|name| LineName::from(name.as_str())).collect();
            }
            if let Some(transports) = &mask.highlight {
                l.highlight = transports
                    .iter()
                    .map(|name| {
                        name.parse::<Transport>()
                            .map_err(|_| McpError::InvalidParams(format!("unknown transport {name}")))
                    })
                    .collect::<Result<_, _>>()?;
            }
            if let Some(v) = mask.stations {
                l.stations = v;
            }
            if let Some(v) = mask.labels {
                l.labels = v;
            }
            if let Some(v) = mask.numbers {
                l.numbers = v;
            }
            if let Some(v) = mask.scores {
                l.scores = v;
            }
            if let Some(mode) = &mask.label_mode {
                l.label_mode = mode.parse().map_err(|_| McpError::InvalidParams(format!("unknown label_mode {mode}")))?;
            }
            if let Some(mode) = &mask.landmark_mode {
                l.landmark_mode = mode.parse().map_err(|_| McpError::InvalidParams(format!("unknown landmark_mode {mode}")))?;
            }
            if let Some(v) = mask.compass {
                l.compass = v;
            }
            if let Some(v) = mask.bridges {
                l.bridges = v;
            }
        }
        scene.update(Rc::new(state.borrow().clone()));
        Ok(CommandResult::LayersApplied)
    })
    .unwrap_or(Err(McpError::StateUnavailable))
}

/// Hands the loaded map to the game host so `request_move` can
/// validate against it. Called once at scene load; the `Rc` is
/// cloned into an `Arc` so the host stays `Send`.
pub fn set_game_map(map: Rc<GameMap>) {
    with_game_host(|host| host.set_map(Arc::new(map.as_ref().clone())));
}

/// Publishes the current session snapshot (`move_log`, `game_config`,
/// resource subscribers). Safe to call unconditionally.
pub fn publish_game(session: Option<GameSession>) {
    MCP_STATE_WATCH.with(|slot| {
        let Some(tx) = slot.borrow().as_ref().cloned() else { return };
        tx.send_modify(|doc| doc.game = session);
    });
}

/// Publishes the current anchor/surface snapshot for MCP consumers.
/// Safe to call unconditionally - without [`attach`] the slot is
/// `None` and this is a no-op.
pub fn publish_state(anchor: &AnchorState, surface: Option<&SurfaceMm>) {
    MCP_STATE_WATCH.with(|slot| {
        let Some(tx) = slot.borrow().as_ref().cloned() else { return };
        let snapshot = AnchorStateSnapshot::captured(anchor, surface);
        tx.send_modify(|doc| doc.anchor = snapshot);
    });
}

/// Publishes the loaded map's static facts (`map_info` tool).
pub fn publish_map(info: MapInfo) {
    MCP_STATE_WATCH.with(|slot| {
        let Some(tx) = slot.borrow().as_ref().cloned() else { return };
        tx.send_modify(|doc| doc.map = Some(info));
    });
}
