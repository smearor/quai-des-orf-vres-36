//! `GameSession` - the rules engine: deal, move validation, turn
//! cycle, win conditions, undo/redo.
//!
//! A command-centric state machine: every input path (pawn, tap,
//! keypad, MCP) produces a [`MoveRequest`] and passes the same
//! validation pipeline - phase, turn, ticket, route, occupancy -
//! before [`GameSession::apply`] mutates state. `past`/`future` give
//! chess-style undo/redo with a physical [`crate::RestorationPlan`].
//!
//! The engine splits across a directory module (chess-style):
//! [`validate`] owns the ordered checks, [`apply`] owns mutation and
//! follow-up transitions; this file keeps the public surface.

mod apply;
mod undo;
mod validate;

#[cfg(test)]
mod tests;

use qdo36_map::GameMap;
use qdo36_map::StationId;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_material::SeatCount;
use qdo36_moves::GameLog;
use qdo36_moves::GamePhase;
use qdo36_moves::Hop;
use qdo36_moves::MoveError;
use qdo36_moves::MoveOutcome;
use qdo36_moves::MoveRequest;
use rand::Rng;
use rand::seq::IndexedRandom;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::GameConfig;
use crate::GameState;
use crate::SessionError;

/// One running (or finished) game with its undo history.
///
/// The session owns every invariant: the hidden `fugitive_pos` only
/// leaves through projections, `allow_undo` gates `undo`/`redo`, and
/// turn order is enforced by `active` - never by callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    /// Configuration fixed at setup.
    pub config: GameConfig,
    /// Current state.
    pub state: GameState,
    /// Undo stack - one entry per committed move.
    pub past: Vec<GameState>,
    /// Redo stack - repopulated by undo.
    pub future: Vec<GameState>,
}

impl GameSession {
    /// Deals a new game: validates the config, samples start stations
    /// without replacement, seats detectives in pawn order and hides
    /// Fugitive on the last dealt station. The session stays in
    /// [`GamePhase::Deal`] until [`Self::begin`].
    ///
    /// `rng` is injected so deals are reproducible - tests and the
    /// balancing simulator seed it deterministically.
    pub fn deal(config: GameConfig, map: &GameMap, rng: &mut impl Rng) -> Result<Self, SessionError> {
        config.validate()?;
        let seats = SeatCount::for_game(config.detectives);
        let starts: Vec<StationId> = map.start_stations().map(|station| station.id).collect();
        let dealt: Vec<StationId> = starts.sample(rng, seats.as_usize()).copied().collect();
        if dealt.len() < seats.as_usize() {
            return Err(SessionError::TooFewStartStations {
                needed: seats,
                available: SeatCount::of_len(starts.len()),
            });
        }
        // Deal layout: one station per detective, the last one hides
        // Fugitive.
        let positions: BTreeMap<PawnId, StationId> = config.detectives.pawns().zip(dealt.iter().copied()).collect();
        let Some(&fugitive_pos) = dealt.last() else {
            return Err(SessionError::TooFewStartStations {
                needed: seats,
                available: SeatCount::of_len(0),
            });
        };
        let mut tickets = BTreeMap::new();
        for pawn in config.detectives.pawns() {
            tickets.insert(Role::Detective(pawn), config.detective_stock);
        }
        tickets.insert(Role::Fugitive, config.fugitive_stock);
        Ok(Self {
            config,
            state: GameState {
                positions,
                fugitive_pos,
                tickets,
                round: RoundNumber::new(1),
                active: Role::Fugitive,
                phase: GamePhase::Deal,
                log: GameLog::default(),
                outcome: None,
            },
            past: Vec::new(),
            future: Vec::new(),
        })
    }

    /// Leaves the Deal phase; Fugitive moves first (round 1).
    pub fn begin(&mut self) -> Result<(), MoveError> {
        if self.state.phase != GamePhase::Deal {
            return Err(MoveError::WrongPhase {
                expected: GamePhase::Deal,
                actual: self.state.phase,
            });
        }
        self.state.phase = GamePhase::InProgress;
        Ok(())
    }

    /// Validates and applies a move - the single entry every input
    /// path funnels through. Validation runs in a fixed order so the
    /// reported [`MoveError`] is deterministic:
    /// phase → turn → ticket → route → occupancy → apply.
    pub fn request_move(&mut self, map: &GameMap, request: &MoveRequest) -> Result<MoveOutcome, MoveError> {
        self.validate_phase()?;
        self.validate_turn(request.actor)?;
        self.validate_ticket(request)?;
        self.validate_hops(map, request)?;
        Ok(self.apply(map, *request))
    }

    /// The stations `role` could legally reach this turn - for UI
    /// move hints and the balancing simulator.
    pub fn legal_destinations(&self, map: &GameMap, role: Role) -> Vec<Hop> {
        let Some(from) = self.position_of(role) else { return Vec::new() };
        let Some(stock) = self.state.tickets.get(&role) else { return Vec::new() };
        map.neighbors(from)
            .iter()
            .filter(|neighbor| self.station_free(neighbor.station))
            .filter(|neighbor| self.can_afford(stock, neighbor.transport, role))
            .map(|neighbor| Hop {
                from,
                to: neighbor.station,
                transport: neighbor.transport,
            })
            .collect()
    }
}
