use qdo36_material::RoundNumber;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;

/// The rounds on which Fugitive's position is revealed to the
/// detectives.
///
/// Classic Scotland Yard reveals on rounds 3, 8, 13, 18 and 24.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevealSchedule {
    /// Round numbers at which Fugitive surfaces.
    pub rounds: BTreeSet<RoundNumber>,
}

impl RevealSchedule {
    /// The classic reveal rounds: 3, 8, 13, 18, 24.
    pub fn classic() -> Self {
        Self {
            rounds: [3, 8, 13, 18, 24].into_iter().map(RoundNumber::new).collect(),
        }
    }

    /// An empty schedule - Fugitive is never revealed until caught.
    pub fn empty() -> Self {
        Self { rounds: BTreeSet::new() }
    }

    /// Builds a schedule from raw round numbers (dataset/IPC boundary).
    pub fn from_rounds(rounds: impl Iterator<Item = u8>) -> Self {
        Self {
            rounds: rounds.map(RoundNumber::new).collect(),
        }
    }

    /// Returns `true` when the given round reveals Fugitive.
    pub fn is_reveal(&self, round: RoundNumber) -> bool {
        self.rounds.contains(&round)
    }
}

impl Default for RevealSchedule {
    fn default() -> Self {
        Self::classic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_reveals() {
        let schedule = RevealSchedule::classic();
        for (round, expected) in [(1, false), (3, true), (7, false), (8, true), (24, true)] {
            assert_eq!(schedule.is_reveal(RoundNumber::new(round)), expected, "round {round}");
        }
    }
}
