//! `HeadingOffsetDeg` - the tower calibration offset, in degrees.
//!
//! Kept as a degree value in `settings.toml` (a human-editable
//! calibration knob - radians would be hostile there) and converted
//! to [`RotationRad`] where `AnchorConfig` consumes it.

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::RotationRad;
use std::str::FromStr;

/// The offset between the Eiffel tower profile's pin frame and map
/// north, in degrees.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HeadingOffsetDeg(f64);

impl HeadingOffsetDeg {
    /// An offset in degrees.
    pub const fn new(degrees: f64) -> Self {
        Self(degrees)
    }

    /// The same offset in radians, for `AnchorConfig`.
    pub fn to_rotation_rad(self) -> RotationRad {
        RotationRad::new(self.0.to_radians())
    }
}

impl FromStr for HeadingOffsetDeg {
    type Err = std::num::ParseFloatError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim().parse::<f64>().map(Self::new)
    }
}
