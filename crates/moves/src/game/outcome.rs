use qdo36_map::StationId;
use serde::Deserialize;
use serde::Serialize;

/// How a finished game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameOutcome {
    /// A detective landed on Fugitive's station - the hunt is over.
    DetectivesWin {
        /// Station where Fugitive was caught.
        caught_at: StationId,
    },
    /// Fugitive survived all 24 rounds.
    FugitiveEscapes,
}
