//! `GotoStationArgs` - parameters of the `goto_station` tool.

use qdo36_map::StationId;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Centers the view on one station.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GotoStationArgs {
    /// Station id to center on.
    #[schemars(with = "u16")]
    pub station: StationId,
    /// Optional zoom factor (0.5-5.0); omit to keep the current
    /// zoom.
    #[serde(default)]
    pub zoom: Option<f64>,
}
