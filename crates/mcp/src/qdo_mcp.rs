//! The `QdoMcp` server type: state snapshots plus subscribers.

use rmcp::RoleServer;
use rmcp::service::Peer;
use serde_json::Value;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::watch::Receiver;

/// `(uri, peer)` pairs that subscribed via `resources/subscribe` and
/// receive `notifications/resources/updated` on each state change.
pub(super) type ResourceSubscribers = Arc<Mutex<Vec<(String, Peer<RoleServer>)>>>;

/// MCP server fronting the app's shared state snapshot.
///
/// Reads come straight from the latest [`watch::Receiver`] document
/// the app publishes - the server never touches GTK state (WP05 R5).
#[derive(Clone)]
pub(super) struct QdoMcp {
    /// Latest published state document.
    pub(super) watch: Receiver<Value>,
    /// Live resource subscriptions; see [`ResourceSubscribers`].
    pub(super) subscribers: ResourceSubscribers,
}

impl QdoMcp {
    /// The current state document.
    pub(super) fn snapshot(&self) -> Value {
        self.watch.borrow().clone()
    }
}
