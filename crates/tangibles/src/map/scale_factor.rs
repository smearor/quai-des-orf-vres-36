//! `ScaleFactor` - a unitless render multiplier (1.0 = neutral).

use serde::Deserialize;
use serde::Serialize;

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
