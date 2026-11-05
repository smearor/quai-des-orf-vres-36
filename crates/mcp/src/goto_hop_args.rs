//! `GotoHopArgs` - parameters of the `goto_hop` tool.

use qdo36_map::StationId;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Frames a hop: the view centers between the two stations and
/// zooms so both markers (and the route between) fit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GotoHopArgs {
    /// First station id of the hop.
    #[schemars(with = "u16")]
    pub from: StationId,
    /// Second station id of the hop.
    #[schemars(with = "u16")]
    pub to: StationId,
    /// Optional zoom factor (0.5-5.0) overriding the auto-fit;
    /// omit to zoom so the hop fills ~55% of the window.
    #[serde(default)]
    pub zoom: Option<f64>,
}
