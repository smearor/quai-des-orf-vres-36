use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A game round (1-based).
///
/// Scotland Yard runs at most 24 rounds (`DEFAULT_MAX`), but the actual
/// game length is configurable per city - the session owns the game
/// clock and enforces the configured maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoundNumber(u8);

impl RoundNumber {
    /// The first round.
    pub const FIRST: RoundNumber = RoundNumber(1);

    /// The default game length (the Scotland Yard classic).
    pub const DEFAULT_MAX: RoundNumber = RoundNumber(24);

    /// Creates a round number (unchecked - the session enforces the
    /// configured maximum).
    pub const fn new(round: u8) -> Self {
        Self(round)
    }

    /// The numeric round.
    pub const fn number(self) -> u8 {
        self.0
    }

    /// The following round (uncapped increment).
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl fmt::Display for RoundNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
