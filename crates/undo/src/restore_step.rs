use qdo36_map::StationId;
use qdo36_material::PawnId;
use serde::Deserialize;
use serde::Serialize;

/// One physical marker move that restores part of a previous position.
///
/// The analog of chess's `MoveBack`/`Restore` steps - simpler, because
/// Scotland Yard has no captures or promotions: an undone pawn simply
/// travels back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreStep {
    /// The pawn to move back.
    pub pawn: PawnId,
    /// Where the pawn currently stands.
    pub from: StationId,
    /// Where the pawn must go to restore the older position.
    pub to: StationId,
}
