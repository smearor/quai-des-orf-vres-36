//! `ScaleFactor` - a unitless render multiplier (1.0 = neutral).

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::ScaleFactor as CommonsScaleFactor;
use std::ops::Mul;

/// A positive render scaling factor shared by all render-param
/// knobs and the spec structs they feed.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScaleFactor(CommonsScaleFactor);

impl ScaleFactor {
    /// The neutral factor (no scaling).
    pub const ONE: ScaleFactor = ScaleFactor(CommonsScaleFactor::ONE);

    /// A factor; clamps non-finite and non-positive input to `ONE`.
    pub fn new(factor: f64) -> Self {
        if factor.is_finite() && factor > 0.0 {
            Self(CommonsScaleFactor::new(factor))
        } else {
            Self::ONE
        }
    }

    /// The factor `after / before` - `ONE` for a `before` span at
    /// or below ~1 px (a degenerate pinch never scales).
    pub fn between(before: Pixels, after: Pixels) -> ScaleFactor {
        Self::new(CommonsScaleFactor::between(before, after).as_f64())
    }

    /// The bare number, for arithmetic.
    pub const fn as_f64(self) -> f64 {
        self.0.as_f64()
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self::ONE
    }
}

impl Mul<ScaleFactor> for Millimeters {
    type Output = Millimeters;

    /// Scales the millimeter length by the render factor (`len * factor`).
    fn mul(self, factor: ScaleFactor) -> Millimeters {
        self * factor.as_f64()
    }
}
