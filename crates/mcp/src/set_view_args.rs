//! `SetViewArgs` - parameters of the `set_view` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Partial view update; every field is optional.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SetViewArgs {
    /// Zoom factor (0.5-5.0, 1.0 = map-mm on surface-mm); omit to keep.
    #[serde(default)]
    pub zoom: Option<f64>,
    /// Display rotation in quarter turns (0-3); omit to keep.
    #[serde(default)]
    pub rotate: Option<u8>,
    /// Pan offset in surface-millimeters `[dx, dy]`; omit to keep.
    #[serde(default)]
    pub pan_mm: Option<[f64; 2]>,
    /// Compass rose position in map-millimeters `[x, y]`; omit to keep.
    #[serde(default)]
    pub compass_mm: Option<[f64; 2]>,
}
