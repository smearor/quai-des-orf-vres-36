use crate::MoveKind;
use qdo36_material::Role;
use qdo36_material::TicketKind;
use serde::Deserialize;
use serde::Serialize;

/// A requested move, before validation.
///
/// Sources are equivalent: a physical pawn placement, a tap on a
/// station, the Corbeau keypad behind the shield, or an MCP agent's
/// `request_move` call - all produce the same `MoveRequest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRequest {
    /// Who wants to move.
    pub actor: Role,
    /// Ticket spent for the move.
    pub ticket: TicketKind,
    /// Shape and destination of the move.
    pub kind: MoveKind,
}
