//! `RotationDeg` - a measured orientation in degrees, for MCP
//! payloads (radians live inside `RotationRad`).

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::RotationRad;

/// An orientation in degrees - the wire-friendly unit for tool
/// replies.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RotationDeg(f64);

impl RotationDeg {
    /// An orientation in degrees.
    pub const fn new(degrees: f64) -> Self {
        Self(degrees)
    }
}

impl From<RotationRad> for RotationDeg {
    fn from(rotation: RotationRad) -> Self {
        Self::new(rotation.as_f64().to_degrees())
    }
}
