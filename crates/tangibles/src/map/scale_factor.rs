//! `ScaleFactor` - a unitless render multiplier (1.0 = neutral).

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Pixels;

/// A positive render scaling factor shared by all render-param
/// knobs and the spec structs they feed.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScaleFactor(f64);

impl ScaleFactor {
    /// The neutral factor (no scaling).
    pub const ONE: ScaleFactor = ScaleFactor(1.0);

    /// A factor; clamps non-finite and non-positive input to `ONE`.
    pub fn new(factor: f64) -> Self {
        if factor.is_finite() && factor > 0.0 { Self(factor) } else { Self::ONE }
    }

    /// The factor `after / before` - `ONE` for a `before` span at
    /// or below ~1 px (a degenerate pinch never scales).
    pub fn between(before: Pixels, after: Pixels) -> ScaleFactor {
        /// Base spans at or below this never pinch (division guard).
        const MIN_BASE: Pixels = Pixels::new(1.0);
        if before > MIN_BASE { Self::new(after / before) } else { Self::ONE }
    }

    /// The bare number, for arithmetic.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self::ONE
    }
}
