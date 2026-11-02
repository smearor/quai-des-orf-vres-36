//! Game-domain MCP server for Quai des Orfèvres 36.
//!
//! rmcp server over Streamable HTTP exposing the app's live state
//! snapshot (anchor pose, map info) to agents - later: role-scoped
//! agent play. The server reads a shared `watch` document the app
//! publishes; it never touches GTK (WP05 R5).

mod anchor_state_snapshot;
mod mcp_error;
mod qdo_mcp;
mod rotation_deg;
mod server_handler;
mod spawn;

pub use anchor_state_snapshot::AnchorStateSnapshot;
pub use mcp_error::McpError;
pub use rotation_deg::RotationDeg;
pub use spawn::spawn_http;

/// URI of the live state resource (`resources/read` +
/// `resources/subscribe` stream).
pub const STATE_URI: &str = "qdo36://state";
