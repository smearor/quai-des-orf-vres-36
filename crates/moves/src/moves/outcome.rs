//! `MoveOutcome` - what an accepted [`MoveRequest`] did to the game.

use crate::GameOutcome;
use crate::MoveRecord;
use serde::Deserialize;
use serde::Serialize;

/// The result of an applied move - validation failures are
/// [`crate::MoveError`] values, not outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MoveOutcome {
    /// The move was applied; play continues.
    Applied(MoveRecord),
    /// The move was applied and the game ended.
    Finished {
        /// The move that ended the game.
        record: MoveRecord,
        /// How it ended.
        outcome: GameOutcome,
    },
}
