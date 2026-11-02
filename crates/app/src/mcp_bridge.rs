//! MCP bridge: publishes the app state snapshot the `qdo36-mcp`
//! endpoint serves.
//!
//! The GTK side only *writes* a typed document into a `watch` channel;
//! the MCP thread reads it independently - no GTK access from the
//! server, ever (WP05 R5).

use qdo36_mcp::AnchorStateSnapshot;
use qdo36_mcp::MapInfo;
use qdo36_mcp::StateSnapshot;
use qdo36_mcp::spawn_http;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::SurfaceMm;
use std::cell::RefCell;
use std::net::SocketAddr;
use tokio::sync::watch::Sender;
use tokio::sync::watch::channel;
use tracing::info;

thread_local! {
    /// Latest-state channel feeding the MCP `qdo36://state` resource
    /// and its subscribers; installed by [`attach`].
    static MCP_STATE_WATCH: RefCell<Option<Sender<StateSnapshot>>> = const { RefCell::new(None) };
}

/// Starts the MCP Streamable-HTTP endpoint on `addr` and installs the
/// state channel. No-op friendly: the app is unchanged when
/// `--mcp-listen` is not passed.
pub fn attach(addr: SocketAddr) {
    let (watch_tx, watch_rx) = channel(StateSnapshot::default());
    MCP_STATE_WATCH.with(|slot| *slot.borrow_mut() = Some(watch_tx));
    spawn_http(watch_rx, addr);
    info!(%addr, "MCP endpoint attached");
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
