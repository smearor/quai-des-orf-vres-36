//! The `#[tool_router]` surface: introspection tools reading the
//! shared state snapshot.

use rmcp::ErrorData;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;

use super::qdo_mcp::QdoMcp;
use crate::McpError;

#[tool_router(vis = "pub(super)")]
impl QdoMcp {
    /// Live anchor facts: detection flag, centroid and orientation.
    #[tool(description = "Get the anchor state: whether the Eiffel tower is detected, its centroid in device mm and its orientation in degrees.")]
    async fn anchor_state(&self) -> Result<CallToolResult, ErrorData> {
        self.reply(&self.snapshot().anchor)
    }

    /// Static facts of the loaded city map.
    #[tool(description = "Get map info: city name, station/edge counts, map bounds in mm and the anchor station position.")]
    async fn map_info(&self) -> Result<CallToolResult, ErrorData> {
        let info = self.snapshot().map.ok_or(McpError::StateUnavailable)?;
        self.reply(&info)
    }
}

impl QdoMcp {
    /// Serializes a reply as pretty JSON text content.
    fn reply(&self, value: &impl serde::Serialize) -> Result<CallToolResult, ErrorData> {
        let text = serde_json::to_string_pretty(value).map_err(McpError::from)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
    }
}
