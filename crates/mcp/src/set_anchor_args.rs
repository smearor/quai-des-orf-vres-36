//! `SetAnchorArgs` - parameters of the `set_anchor` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Places a virtual anchor on the surface; bypasses the
/// "place the tower" setup for headless/table-less runs.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SetAnchorArgs {
    /// Anchor center in surface-millimeters `[x, y]`.
    pub center_mm: [f64; 2],
    /// Anchor orientation in radians; omit for upright.
    #[serde(default)]
    pub rotation_rad: Option<f64>,
}
