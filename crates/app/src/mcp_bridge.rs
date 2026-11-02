//! MCP bridge: publishes the app state snapshot the `qdo36-mcp`
//! endpoint serves.
//!
//! The GTK side only *writes* a JSON document into a `watch` channel
//! (`publish_state`); the MCP thread reads it independently - no GTK
//! access from the server, ever (WP05 R5).

use qdo36_tangibles::AnchorState;
use qdo36_tangibles::SurfaceMm;
use serde_json::Value;
use std::cell::RefCell;
use std::net::SocketAddr;
use tracing::info;

thread_local! {
    /// Latest-state channel feeding the MCP `qdo36://state` resource
    /// and its subscribers; installed by [`attach`].
    static MCP_STATE_WATCH: RefCell<Option<tokio::sync::watch::Sender<Value>>> = const { RefCell::new(None) };
}

/// Starts the MCP Streamable-HTTP endpoint on `addr` and installs the
/// state channel. No-op friendly: the app is unchanged when
/// `--mcp-listen` is not passed.
pub fn attach(addr: SocketAddr) {
    let (watch_tx, watch_rx) = tokio::sync::watch::channel(serde_json::json!({}));
    MCP_STATE_WATCH.with(|slot| *slot.borrow_mut() = Some(watch_tx));
    qdo36_mcp::spawn_http(watch_rx, addr);
    info!(%addr, "MCP endpoint attached");
}

/// Publishes the current anchor/surface snapshot for MCP consumers.
/// Safe to call unconditionally - without [`attach`] the slot is
/// `None` and this is a no-op.
pub fn publish_state(anchor: &AnchorState, surface: Option<&SurfaceMm>) {
    MCP_STATE_WATCH.with(|slot| {
        let Some(tx) = slot.borrow().as_ref().cloned() else { return };
        let snapshot = qdo36_mcp::AnchorStateSnapshot::captured(anchor, surface);
        if let Ok(value) = serde_json::to_value(snapshot) {
            let _ = tx.send(value);
        }
    });
}
