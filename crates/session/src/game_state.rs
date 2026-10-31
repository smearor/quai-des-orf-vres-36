use qdo36_map::StationId;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_material::TicketCounts;
use qdo36_moves::GameLog;
use qdo36_moves::GameOutcome;
use qdo36_moves::GamePhase;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

/// The complete state of one game snapshot.
///
/// `corbeau_pos` is the hidden truth - it must never leak into public
/// projections (HUD, public MCP state); the private view is served
/// only inside the shield zone / to the Corbeau token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    /// Detective pawn positions.
    pub positions: BTreeMap<PawnId, StationId>,
    /// Corbeau's true position - hidden from the public view.
    pub corbeau_pos: StationId,
    /// Ticket inventories per role.
    pub tickets: BTreeMap<Role, TicketCounts>,
    /// Current round.
    pub round: RoundNumber,
    /// Whose turn it is.
    pub active: Role,
    /// Game lifecycle phase.
    pub phase: GamePhase,
    /// Committed move log.
    pub log: GameLog,
    /// Set once the game ends.
    pub outcome: Option<GameOutcome>,
}
