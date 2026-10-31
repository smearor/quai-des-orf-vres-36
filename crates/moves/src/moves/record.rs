use crate::MoveKind;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_material::TicketKind;
use serde::Deserialize;
use serde::Serialize;

/// A committed move in the game log.
///
/// `destination` is the truth (where Corbeau actually went); the public
/// projection [`crate::GameLog::corbeau_public_view`] redacts it until
/// the round is a reveal round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRecord {
    /// Round in which the move happened.
    pub round: RoundNumber,
    /// Who moved.
    pub actor: Role,
    /// Ticket spent.
    pub ticket: TicketKind,
    /// Shape and destination of the move.
    pub kind: MoveKind,
}
