//! `Variant` - a house's texture variant inside its width class.

use crate::map::blocks::noise::noise;

/// Texture variant `0..=MAX` within the unit-width class - styles
/// pick fills and hatches by `(units, variant)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variant(u8);

impl Variant {
    /// Highest texture variant.
    pub const MAX: u8 = 3;
    /// The variant landmark volumes draw with.
    pub const LANDMARK: Variant = Variant(3);

    /// A validated variant - `None` above `MAX`.
    pub fn new(variant: u8) -> Option<Self> {
        (variant <= Self::MAX).then_some(Self(variant))
    }

    /// The variant roll in `0..=MAX`.
    pub fn roll(roll: f64) -> Self {
        Self((roll * f64::from(Self::MAX + 1)).min(f64::from(Self::MAX)) as u8)
    }

    /// The seeded texture variant for the slot `slot` of edge
    /// `edge` in ring `ring` - rolls its `0..=MAX` on noise
    /// channel 2.
    pub fn of_seed(seed: u64, ring: usize, edge: usize, slot: usize) -> Self {
        Self::roll(noise(seed, ring, edge, slot, 2))
    }

    /// The raw index.
    pub fn as_u8(self) -> u8 {
        self.0
    }

    /// The index as `f64` for shade math.
    pub fn as_f64(self) -> f64 {
        f64::from(self.0)
    }
}
