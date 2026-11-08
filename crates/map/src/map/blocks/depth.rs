//! `Depth` - how deep a house reaches into its block (mm).

/// House depth in map millimeters. `roll` quantizes into `STEP`
/// increments inside the caller's `min..=max`; `new` accepts any
/// positive depth (the valid range is configured per
/// `BuildingParams`, not a const).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Depth(f64);

impl Depth {
    /// The depth quantum rolled values snap to (mm).
    pub const STEP: f64 = 0.5;
    /// The fixed depth landmark volumes draw with (mm).
    pub const LANDMARK: Depth = Depth(3.0);

    /// A depth in mm.
    pub const fn new(depth: f64) -> Self {
        Self(depth)
    }

    /// The depth a house rolls: `min + STEP * k` inside
    /// `min..=max` mm.
    pub fn roll(roll: f64, min: Depth, max: Depth) -> Self {
        let steps = ((max.0 - min.0) / Self::STEP).round().max(0.0) as u32;
        Self(min.0 + Self::STEP * f64::from(steps).min(roll * f64::from(steps + 1)).floor())
    }

    /// The depth in mm for geometry math.
    pub fn as_f64(self) -> f64 {
        self.0
    }
}

impl std::fmt::Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
