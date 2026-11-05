//! `LayerMask` - partial layer-visibility update
//! (`set_layers` command and args share it).

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Per-layer visibility; `null` keeps the current flag.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct LayerMask {
    /// Water polylines.
    #[serde(default)]
    pub water: Option<bool>,
    /// Backdrop landmark pins.
    #[serde(default)]
    pub landmarks: Option<bool>,
    /// Taxi edges.
    #[serde(default)]
    pub taxi: Option<bool>,
    /// Bus edges.
    #[serde(default)]
    pub bus: Option<bool>,
    /// Metro edges (all lines).
    #[serde(default)]
    pub metro: Option<bool>,
    /// Batobus Seine edges.
    #[serde(default)]
    pub batobus: Option<bool>,
    /// Underground secret-route edges.
    #[serde(default)]
    pub underground: Option<bool>,
    /// Taxi/bus hops shared by several transports drawn side by
    /// side in each transport's color.
    #[serde(default)]
    pub parallel_transports: Option<bool>,
    /// Transports drawn in their own colors (the rest render as
    /// neutral edges); replaces the highlight set when present.
    #[serde(default)]
    pub highlight: Option<Vec<String>>,
    /// Metro lines hidden by name (e.g. `["M4","M13"]`); replaces
    /// the hidden set when present.
    #[serde(default)]
    pub hide_lines: Option<Vec<String>>,
    /// Station discs.
    #[serde(default)]
    pub stations: Option<bool>,
    /// Station name labels.
    #[serde(default)]
    pub labels: Option<bool>,
    /// Station number labels inside the disc.
    #[serde(default)]
    pub numbers: Option<bool>,
    /// Strategic-importance score labels beside stations.
    #[serde(default)]
    pub scores: Option<bool>,
    /// Label orientation: `radial` faces the nearest table edge
    /// (tabletop default), `screen` draws every label upright.
    #[serde(default)]
    pub label_mode: Option<String>,
    /// The compass rose.
    #[serde(default)]
    pub compass: Option<bool>,
    /// Bridge-station markers (dataset debugging).
    #[serde(default)]
    pub bridges: Option<bool>,
}
