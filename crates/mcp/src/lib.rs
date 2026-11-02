//! Game-domain MCP server for Quai des Orfèvres 36.
//!
//! rmcp server over Streamable HTTP exposing the app's live state
//! snapshot (anchor pose, map info) to agents - later: role-scoped
//! agent play. The server reads a shared `watch` document the app
//! publishes; it never touches GTK (WP05 R5).

mod anchor_state_snapshot;
pub mod command;
mod game_request;
mod game_state_view;
mod hop_args;
mod join_game_args;
mod map_info;
mod mcp_error;
mod new_game_args;
mod qdo_mcp;
mod request_move_args;
mod role_claim;
mod rotation_deg;
mod screenshot_reply;
mod server_handler;
mod set_config_args;
mod spawn;
mod state_snapshot;
mod token_args;
mod tools;

pub use anchor_state_snapshot::AnchorStateSnapshot;
pub use command::CommandKind;
pub use command::CommandReply;
pub use command::CommandResult;
pub use game_request::GameRequest;
pub use game_state_view::GameStateView;
pub use hop_args::HopArgs;
pub use join_game_args::JoinGameArgs;
pub use map_info::MapInfo;
pub use mcp_error::McpError;
pub use new_game_args::NewGameArgs;
pub use request_move_args::RequestMoveArgs;
pub use role_claim::RoleClaim;
pub use rotation_deg::RotationDeg;
pub use screenshot_reply::ScreenshotReply;
pub use set_config_args::SetConfigArgs;
pub use spawn::spawn_http;
pub use state_snapshot::StateSnapshot;
pub use token_args::TokenArgs;

/// URI of the live state resource (`resources/read` +
/// `resources/subscribe` stream).
pub const STATE_URI: &str = "qdo36://state";
