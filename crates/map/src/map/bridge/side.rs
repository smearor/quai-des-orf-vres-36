//! `DeckSide` - which side of a bridge deck a position lies on.

/// Which bank-side of a `BridgeDeck` a position lies on, measured
/// along the deck `axis` (perpendicular to the river).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckSide {
    /// Same direction as the deck axis.
    Along,
    /// Against the deck axis.
    Against,
    /// Exactly on the deck axis.
    Center,
}

impl DeckSide {
    /// The axis factor the side stands for: `+1`/`0`/`-1`.
    pub const fn factor(self) -> f64 {
        match self {
            Self::Along => 1.0,
            Self::Against => -1.0,
            Self::Center => 0.0,
        }
    }
}
