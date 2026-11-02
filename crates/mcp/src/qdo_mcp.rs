//! The `QdoMcp` server type: state snapshots plus subscribers.

use async_channel::Sender;
use rmcp::RoleServer;
use rmcp::service::Peer;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::watch::Receiver;

use crate::GameRequest;
use crate::McpError;
use crate::StateSnapshot;
use crate::command::CommandKind;
use crate::command::CommandReply;

/// `(uri, peer)` pairs that subscribed via `resources/subscribe` and
/// receive `notifications/resources/updated` on each state change.
pub(super) type ResourceSubscribers = Arc<Mutex<Vec<(String, Peer<RoleServer>)>>>;

/// MCP server fronting the app's shared state snapshot.
///
/// Reads come straight from the latest [`watch::Receiver`] document
/// the app publishes - the server never touches GTK state (WP05 R5).
/// Write commands travel an `async_channel` of [`GameRequest`]s into
/// the app's session owner; replies come back on a oneshot.
#[derive(Clone)]
pub(super) struct QdoMcp {
    /// Latest published state document.
    pub(super) watch: Receiver<StateSnapshot>,
    /// Command channel into the app-side game host.
    pub(super) request_tx: Sender<GameRequest>,
    /// Live resource subscriptions; see [`ResourceSubscribers`].
    pub(super) subscribers: ResourceSubscribers,
}

impl QdoMcp {
    /// The current state document.
    pub(super) fn snapshot(&self) -> StateSnapshot {
        self.watch.borrow().clone()
    }

    /// Sends a command to the game host and awaits the typed reply.
    pub(super) async fn command(&self, kind: CommandKind) -> Result<CommandReply, McpError> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.request_tx
            .send(GameRequest {
                command: kind,
                reply: reply_tx,
            })
            .await
            .map_err(|_| McpError::ChannelClosed)?;
        reply_rx.await.map_err(|_| McpError::NoReply)
    }
}
