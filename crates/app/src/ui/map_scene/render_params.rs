//! `RenderParams` - global display factors tuned live via MCP and
//! the on-screen controls (independent of the map camera zoom).

use qdo36_tangibles::ScaleFactor;

/// Global render factors, each `1.0` neutral.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderParams {
    /// Station disc radius factor.
    pub station_scale: ScaleFactor,
    /// Landmark pin height factor.
    pub landmark_scale: ScaleFactor,
    /// Metro line-badge height factor.
    pub metro_scale: ScaleFactor,
    /// Transport-ring stroke width factor.
    pub ring_width: ScaleFactor,
    /// Label font size factor.
    pub label_scale: ScaleFactor,
}
