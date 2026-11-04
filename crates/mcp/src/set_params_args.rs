//! `SetParamsArgs` - parameters of the `set_params` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Partial render-factor update; every field is optional, `1.0`
/// is neutral.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SetParamsArgs {
    /// Station disc radius factor.
    #[serde(default)]
    pub station_scale: Option<f64>,
    /// Landmark pin size factor.
    #[serde(default)]
    pub landmark_scale: Option<f64>,
    /// Metro line-badge size factor.
    #[serde(default)]
    pub metro_scale: Option<f64>,
    /// Transport-ring stroke width factor.
    #[serde(default)]
    pub ring_width: Option<f64>,
    /// Label font size factor.
    #[serde(default)]
    pub label_scale: Option<f64>,
}
