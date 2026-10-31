//! Game configuration: the setup dialog's choices.

pub mod mode;
pub mod preset;

pub use mode::TangibleMode;
pub use preset::RevealPreset;

use crate::SessionError;
use qdo36_map::MapMeta;
use qdo36_material::DetectiveCount;
use qdo36_material::PawnColor;
use qdo36_material::RoundNumber;
use qdo36_moves::RevealSchedule;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;

/// Configuration chosen in the setup dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameConfig {
    /// How many detectives hunt Corbeau (2-5).
    pub detectives: DetectiveCount,
    /// Pawn colors in seat order; `colors.len() == detectives.count()`.
    pub colors: Vec<PawnColor>,
    /// Input mode for pawns.
    pub mode: TangibleMode,
    /// Whether undo/redo is offered (chess's `allow_undo` gate).
    pub allow_undo: bool,
    /// Rounds on which Corbeau's position is revealed.
    pub reveal: RevealPreset,
    /// Maximum game length; Corbeau escapes when reached.
    ///
    /// Defaults to the city's dataset value (Paris: 24); other cities
    /// may ship a different default in their `[map]` meta.
    pub max_rounds: RoundNumber,
}

impl GameConfig {
    /// Builds a config from a map's city defaults.
    pub fn for_map(meta: &MapMeta) -> Self {
        Self {
            max_rounds: meta.max_rounds,
            reveal: RevealPreset::from_city(&meta.reveal_rounds),
            ..Self::default()
        }
    }

    /// The effective reveal schedule.
    pub fn reveal_schedule(&self) -> RevealSchedule {
        self.reveal.schedule()
    }

    /// Cross-field validation beyond the per-type checks.
    ///
    /// # Errors
    ///
    /// - [`SessionError::ColorCountMismatch`] when `colors` does not
    ///   match `detectives`
    /// - [`SessionError::DuplicatePawnColor`] on repeated colors
    /// - [`SessionError::InvalidMaxRounds`] for a zero game length
    /// - [`SessionError::RevealBeyondMaxRounds`] for reveals after the
    ///   last round
    pub fn validate(&self) -> Result<(), SessionError> {
        if self.max_rounds.number() == 0 {
            return Err(SessionError::InvalidMaxRounds(self.max_rounds));
        }
        if let Some(round) = self.reveal_schedule().rounds.iter().find(|r| **r > self.max_rounds) {
            return Err(SessionError::RevealBeyondMaxRounds {
                round: *round,
                max_rounds: self.max_rounds,
            });
        }
        let mut seen = BTreeSet::new();
        for color in &self.colors {
            if !seen.insert(*color) {
                return Err(SessionError::DuplicatePawnColor(*color));
            }
        }
        if self.colors.len() != self.detectives.count() as usize {
            return Err(SessionError::ColorCountMismatch {
                given: self.colors.len(),
                expected: self.detectives,
            });
        }
        Ok(())
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            detectives: DetectiveCount::try_from(4).unwrap_or_else(|_| DetectiveCount::try_from(DetectiveCount::MIN).unwrap_or_else(|e| panic!("{e}"))),
            colors: vec![PawnColor::Blue, PawnColor::Red, PawnColor::Green, PawnColor::Yellow],
            mode: TangibleMode::TangiblesRendered,
            allow_undo: true,
            reveal: RevealPreset::Classic,
            max_rounds: RoundNumber::DEFAULT_MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qdo36_map::City;
    use qdo36_map::GameMap;

    #[test]
    fn paris_defaults_map_to_classic_preset() {
        let map = GameMap::for_city(City::Paris).unwrap_or_else(|e| panic!("{e}"));
        let config = GameConfig::for_map(map.meta());
        assert_eq!(config.reveal, RevealPreset::Classic);
        assert_eq!(config.max_rounds, RoundNumber::DEFAULT_MAX);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn berlin_defaults_map_to_custom_preset() {
        let map = GameMap::for_city(City::Berlin).unwrap_or_else(|e| panic!("{e}"));
        let config = GameConfig::for_map(map.meta());
        assert!(matches!(config.reveal, RevealPreset::Custom(_)));
        assert_eq!(config.max_rounds, map.meta().max_rounds);
        assert!(config.validate().is_ok());
    }
}
