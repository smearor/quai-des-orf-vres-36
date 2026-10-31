use qdo36_material::RoundNumber;
use qdo36_moves::RevealSchedule;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;

/// The setup dialog's reveal-schedule choice.
///
/// `Custom` opens a second dialog editing the schedule - the resolved
/// [`RevealSchedule`] is carried inline so the choice round-trips
/// through config files and MCP unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevealPreset {
    /// The Scotland Yard classic: rounds 3, 8, 13, 18, 24.
    #[default]
    Classic,
    /// Corbeau is never revealed until caught.
    None,
    /// A custom schedule edited in the setup dialog.
    Custom(RevealSchedule),
}

impl RevealPreset {
    /// The preset's effective schedule.
    pub fn schedule(&self) -> RevealSchedule {
        match self {
            Self::Classic => RevealSchedule::classic(),
            Self::None => RevealSchedule::empty(),
            Self::Custom(schedule) => schedule.clone(),
        }
    }

    /// Builds a preset from a city's dataset defaults: a dataset
    /// matching the classic schedule maps to `Classic`, anything else
    /// to `Custom`.
    pub fn from_city(reveal_rounds: &BTreeSet<RoundNumber>) -> Self {
        if *reveal_rounds == RevealSchedule::classic().rounds {
            Self::Classic
        } else {
            Self::Custom(RevealSchedule { rounds: reveal_rounds.clone() })
        }
    }
}
