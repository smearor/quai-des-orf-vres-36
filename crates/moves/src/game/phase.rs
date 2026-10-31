use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Lifecycle phase of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GamePhase {
    /// Start deal and placement ceremony; no moves yet.
    Deal,
    /// The hunt is on.
    InProgress,
    /// The game ended; the log may be replayed.
    Finished,
}

impl fmt::Display for GamePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Deal => "deal",
            Self::InProgress => "in-progress",
            Self::Finished => "finished",
        };
        f.write_str(name)
    }
}
