//! `McpError` - errors mapped onto rmcp `ErrorData` replies.

use qdo36_moves::MoveError;
use qdo36_session::SessionError;
use rmcp::ErrorData;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

/// MCP-side failures.
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum McpError {
    /// A request parameter was wrong (unknown resource, bad args).
    #[error("invalid params: {0}")]
    InvalidParams(String),
    /// Reply serialization failed.
    #[error("serialization failed: {0}")]
    Serialize(String),
    /// The requested state does not exist yet (e.g. no map loaded).
    #[error("state unavailable")]
    StateUnavailable,
    /// No game is dealt yet; the command needs a live session.
    #[error("no active game")]
    NoActiveGame,
    /// The presented token is unknown to this game.
    #[error("bad player token")]
    BadToken,
    /// The claimed role is already taken by another token.
    #[error("role {0} is already taken")]
    RoleTaken(String),
    /// `set_config` is only valid while the game is in the Deal phase.
    #[error("config changes are Deal-phase only")]
    DealPhaseOnly,
    /// The detective seat does not exist in this game.
    #[error("seat does not exist")]
    SeatUnavailable,
    /// The app-side command channel closed (app exiting).
    #[error("game host channel closed")]
    ChannelClosed,
    /// The app dropped the reply without answering.
    #[error("game host dropped the reply")]
    NoReply,
    /// The rules engine rejected the move.
    #[error("move rejected: {0}")]
    Move(String),
    /// Session setup/persistence failed.
    #[error("session error: {0}")]
    Session(String),
}

impl From<serde_json::Error> for McpError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialize(error.to_string())
    }
}

impl From<MoveError> for McpError {
    fn from(error: MoveError) -> Self {
        Self::Move(error.to_string())
    }
}

impl From<SessionError> for McpError {
    fn from(error: SessionError) -> Self {
        Self::Session(error.to_string())
    }
}

impl From<McpError> for ErrorData {
    fn from(error: McpError) -> Self {
        match error {
            McpError::InvalidParams(_) => ErrorData::invalid_params(error.to_string(), None),
            McpError::BadToken | McpError::RoleTaken(_) | McpError::DealPhaseOnly | McpError::SeatUnavailable | McpError::Move(_) => {
                ErrorData::invalid_params(error.to_string(), None)
            }
            _ => ErrorData::internal_error(error.to_string(), None),
        }
    }
}
