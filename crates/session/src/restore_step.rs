//! `RestoreStep` - one physical pawn relocation after undo/redo.

use qdo36_map::StationId;
use qdo36_material::PawnId;
use serde::Deserialize;
use serde::Serialize;

/// One pawn that must physically move between two stations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreStep {
    /// The pawn to move.
    pub pawn: PawnId,
    /// Where it stood after the move that was undone (current spot).
    pub from: StationId,
    /// Where it must be returned to.
    pub to: StationId,
}
