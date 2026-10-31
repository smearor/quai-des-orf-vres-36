use qdo36_map::StationId;
use qdo36_material::RoundNumber;
use qdo36_material::TicketKind;
use serde::Deserialize;
use serde::Serialize;

/// One log entry as seen by the detectives (public view).
///
/// Fugitive's ticket is always public - the detectives *hear* which
/// transport he took - but his destination stays hidden unless the
/// round is a reveal round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicEntry {
    /// Round of the recorded move.
    pub round: RoundNumber,
    /// Ticket spent (always visible).
    pub ticket: TicketKind,
    /// Destination, if this entry is publicly visible.
    ///
    /// Always `Some` for detective moves; `Some` for Fugitive only on
    /// reveal rounds, `None` otherwise.
    pub destination: Option<StationId>,
}
