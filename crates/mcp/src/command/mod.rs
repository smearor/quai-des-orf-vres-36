//! The command channel types crossing the MCP↔app boundary.

pub mod kind;
pub mod reply;

pub use kind::CommandKind;
pub use reply::CommandReply;
pub use reply::CommandResult;
