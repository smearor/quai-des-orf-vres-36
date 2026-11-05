//! `Luminance` - a perceived color luminance (0.0-1.0).

/// A perceived luminance value (ITU-R BT.709 weighted), clamped
/// to `0.0-1.0` on construction. Newtype so a minimum threshold
/// cannot be confused with a percent or channel value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Luminance(f64);

impl Luminance {
    /// Creates a luminance value, clamped to `0.0-1.0`.
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// The numeric value.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}
