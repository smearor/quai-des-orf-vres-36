//! `HeadingOffsetDeg` - the tower calibration offset, in degrees.
//!
//! Kept as a degree value in `settings.toml` (a human-editable
//! calibration knob - radians would be hostile there) and converted
//! to [`RotationRad`] where `AnchorConfig` consumes it.

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Degrees;
use smearor_commons_geometry::RotationRad;
use std::str::FromStr;

/// The offset between the Eiffel tower profile's pin frame and map
/// north, in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HeadingOffsetDeg(Degrees);

impl HeadingOffsetDeg {
    /// An offset in degrees.
    pub const fn new(degrees: f64) -> Self {
        Self(Degrees::new(degrees))
    }

    /// The same offset in radians, for `AnchorConfig`.
    pub fn to_rotation_rad(self) -> RotationRad {
        RotationRad::from(self.0)
    }
}

impl Default for HeadingOffsetDeg {
    /// No calibration offset.
    fn default() -> Self {
        Self(Degrees::ZERO)
    }
}

impl FromStr for HeadingOffsetDeg {
    type Err = std::num::ParseFloatError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim().parse::<f64>().map(Self::new)
    }
}
