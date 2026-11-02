//! `McpError` - errors mapped onto rmcp `ErrorData` replies.

use rmcp::ErrorData;
use thiserror::Error;

/// MCP-side failures.
#[derive(Debug, Error)]
pub enum McpError {
    /// A request parameter was wrong (unknown resource, bad args).
    #[error("invalid params: {0}")]
    InvalidParams(String),
    /// Reply serialization failed.
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The requested state does not exist yet (e.g. no map loaded).
    #[error("state unavailable")]
    StateUnavailable,
}

impl From<McpError> for ErrorData {
    fn from(error: McpError) -> Self {
        match error {
            McpError::InvalidParams(_) => ErrorData::invalid_params(error.to_string(), None),
            McpError::Serialize(_) | McpError::StateUnavailable => ErrorData::internal_error(error.to_string(), None),
        }
    }
}
