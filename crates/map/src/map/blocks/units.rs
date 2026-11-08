//! `Units` - a house's frontage width class (1..=4).

/// Frontage width in units: row houses roll 1-4, corner houses
/// 1-2 per side. Bounds are enforced by `new`; `roll` always
/// produces a valid value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Units(u8);

impl Units {
    /// Narrowest frontage.
    pub const MIN: u8 = 1;
    /// Widest frontage.
    pub const MAX: u8 = 4;
    /// A single unit - landmark volumes and the whole-face
    /// fallback.
    pub const ONE: Units = Units(1);
    /// Two units - the wider corner-house candidate.
    pub const TWO: Units = Units(2);

    /// A validated frontage - `None` outside `MIN..=MAX`.
    pub fn new(units: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX).contains(&units).then_some(Self(units))
    }

    /// The width-class roll: small houses dominate, 4-unit rows
    /// are rare.
    pub fn roll(roll: f64) -> Self {
        let units = if roll < 0.42 {
            1
        } else if roll < 0.68 {
            2
        } else if roll < 0.87 {
            3
        } else {
            4
        };
        Self(units)
    }

    /// One unit narrower, clamped at `MIN` (fit-shrinking rows).
    pub fn saturating_dec(self) -> Self {
        Self(self.0.saturating_sub(1).max(Self::MIN))
    }

    /// The raw count.
    pub fn as_u8(self) -> u8 {
        self.0
    }

    /// The count as `f64` for geometry math.
    pub fn as_f64(self) -> f64 {
        f64::from(self.0)
    }
}

impl std::fmt::Display for Units {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
