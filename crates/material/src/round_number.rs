use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A game round (1-based).
///
/// Scotland Yard runs at most 24 rounds; Corbeau escapes if he is not
/// caught by then.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoundNumber(u8);

impl RoundNumber {
    /// The first round.
    pub const FIRST: RoundNumber = RoundNumber(1);
    /// The last round of a game.
    pub const LAST: RoundNumber = RoundNumber(24);

    /// Creates a round number (unchecked - valid rounds are enforced by
    /// the session, which owns the game clock).
    pub const fn new(round: u8) -> Self {
        Self(round)
    }

    /// The numeric round.
    pub const fn number(self) -> u8 {
        self.0
    }

    /// The next round, saturating at `LAST`.
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1).min(Self::LAST.0))
    }
}

impl fmt::Display for RoundNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
