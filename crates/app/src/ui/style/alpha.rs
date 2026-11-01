//! `Alpha` - a normalized (0.0-1.0) alpha channel value.

/// A normalized alpha value, clamped to `0.0-1.0` on construction.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Alpha(f64);

impl Alpha {
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self(0.0);
    /// Fully opaque.
    pub const OPAQUE: Self = Self(1.0);

    /// Creates an alpha value, clamped to `0.0-1.0`.
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// The numeric value.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl Default for Alpha {
    fn default() -> Self {
        Self::OPAQUE
    }
}
