//! MCP bridge: publishes the app state snapshot the `qdo36-mcp`
//! endpoint serves, and drains the command channel into the
//! app-owned [`GameHost`].
//!
//! The GTK side *writes* a typed document into a `watch` channel and
//! *executes* `GameRequest`s on the main loop; the MCP thread reads
//! the document and forwards commands - no GTK access from the
//! server, ever (WP05 R5).

use async_channel::unbounded;
use glib::MainContext;
use libadwaita as adw;
use qdo36_host::GameHost;
use qdo36_map::GameMap;
use qdo36_mcp::AnchorStateSnapshot;
use qdo36_mcp::CommandKind;
use qdo36_mcp::CommandReply;
use qdo36_mcp::CommandResult;
use qdo36_mcp::GameRequest;
use qdo36_mcp::MapInfo;
use qdo36_mcp::McpError;
use qdo36_mcp::ScreenshotReply;
use qdo36_mcp::StateSnapshot;
use qdo36_mcp::spawn_http;
use qdo36_session::GameSession;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::SurfaceMm;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use tangible_objects_model::Pixels;
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
            // `Screenshot` is an app concern, not a session one - the
            // game host never sees it.
            let reply = if matches!(request.command, CommandKind::Screenshot) {
                mcp_screenshot()
            } else {
                with_game_host(|host| host.handle(&request.command)).unwrap_or(Err(McpError::ChannelClosed))
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
