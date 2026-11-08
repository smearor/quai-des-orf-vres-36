//! `Height` - a house's isometric extrusion factor.

/// Extrusion factor for the isometric house modes: `STANDARD` is
/// the block's base height, rolls quantize into `STEP` increments
/// inside `MIN..=MAX`, landmark volumes use `LANDMARK`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Height(f64);

impl Height {
    /// Flattest rolled roof (factor).
    pub const MIN: f64 = 0.75;
    /// Tallest rolled roof (factor).
    pub const MAX: f64 = 1.75;
    /// The standard height (factor).
    pub const STANDARD: Height = Height(1.0);
    /// The factor landmark volumes draw with.
    pub const LANDMARK: Height = Height(3.0);
    /// The height quantum rolled values snap to (factor).
    const STEP: f64 = 0.25;

    /// A factor.
    pub fn new(height: f64) -> Self {
        Self(height)
    }

    /// The factor a house rolls: `MIN + STEP * k` inside
    /// `MIN..=MAX`.
    pub fn roll(roll: f64) -> Self {
        let steps = ((Self::MAX - Self::MIN) / Self::STEP).round() as u32;
        Self(Self::MIN + Self::STEP * f64::from(steps).min(roll * f64::from(steps + 1)).floor())
    }

    /// The raw factor for extrusion math.
    pub fn as_f64(self) -> f64 {
        self.0
    }
}
