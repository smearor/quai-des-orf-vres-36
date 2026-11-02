//! `GameStateView` - the role-scoped projection a `game_state` call
//! returns to a token holder.

use qdo36_map::StationId;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_material::TicketCounts;
use qdo36_moves::GameOutcome;
use qdo36_moves::GamePhase;
use qdo36_moves::PublicEntry;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

/// What a token holder may see of the running game.
///
/// Detectives get the public projection only; the Fugitive token
/// additionally carries his true position - the same secrecy split
/// the shield zone enforces (WP05 R4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateView {
    /// Game lifecycle phase.
    pub phase: GamePhase,
    /// Current round.
    pub round: RoundNumber,
    /// The caller's seat.
    pub your_role: Role,
    /// Detective pawn positions (public).
    pub positions: BTreeMap<PawnId, StationId>,
    /// Fugitive's true position - `Some` only for the Fugitive token.
    pub fugitive_pos: Option<StationId>,
    /// The caller's remaining ticket stock.
    pub your_stock: TicketCounts,
    /// The public log projection (Fugitive destinations only on
    /// reveal rounds).
    pub log: Vec<PublicEntry>,
    /// Set once the game ended.
    pub outcome: Option<GameOutcome>,
    /// Whether the game accepts undo right now.
    pub can_undo: bool,
    /// Whether the game accepts redo right now.
    pub can_redo: bool,
}
