//! `GameSession` - the rules engine: deal, move validation, turn
//! cycle, win conditions, undo/redo.
//!
//! A command-centric state machine: every input path (pawn, tap,
//! keypad, MCP) produces a [`MoveRequest`] and passes the same
//! validation pipeline - phase, turn, ticket, route, occupancy -
//! before [`GameSession::apply`] mutates state. `past`/`future` give
//! chess-style undo/redo with a physical [`RestorationPlan`].
//!
//! The engine splits across a directory module (chess-style):
//! [`validate`] owns the ordered checks, [`apply`] owns mutation and
//! follow-up transitions; this file keeps the public surface.

mod apply;
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
use crate::RestorationPlan;
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
        let detectives = config.detectives.count() as usize;
        let seats = SeatCount::for_game(config.detectives);
        let starts: Vec<StationId> = map.start_stations().map(|station| station.id).collect();
        let dealt: Vec<StationId> = starts.sample(rng, seats.count() as usize).copied().collect();
        if dealt.len() < seats.count() as usize {
            return Err(SessionError::TooFewStartStations {
                needed: seats,
                available: SeatCount::new(starts.len() as u8),
            });
        }
        let positions: BTreeMap<PawnId, StationId> = dealt
            .iter()
            .take(detectives)
            .enumerate()
            .map(|(seat, station)| (PawnId::new(seat as u8), *station))
            .collect();
        let mut tickets = BTreeMap::new();
        for seat in 0..detectives {
            tickets.insert(Role::Detective(PawnId::new(seat as u8)), config.detective_stock);
        }
        tickets.insert(Role::Fugitive, config.fugitive_stock);
        Ok(Self {
            config,
            state: GameState {
                positions,
                fugitive_pos: dealt[detectives],
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

    /// Whether undo is possible right now (also gated by
    /// `config.allow_undo`).
    pub fn can_undo(&self) -> bool {
        self.config.allow_undo && !self.past.is_empty()
    }

    /// Whether redo is possible right now.
    pub fn can_redo(&self) -> bool {
        self.config.allow_undo && !self.future.is_empty()
    }

    /// Reverts the last move; the returned plan lists the physical
    /// pawn relocations (empty for Fugitive - nothing on the board,
    /// secrecy preserved).
    pub fn undo(&mut self) -> Option<RestorationPlan> {
        if !self.can_undo() {
            return None;
        }
        let previous = self.past.pop()?;
        let plan = RestorationPlan::between(&self.state, &previous);
        self.future.push(std::mem::replace(&mut self.state, previous));
        Some(plan)
    }

    /// Re-applies the last undone move.
    pub fn redo(&mut self) -> Option<RestorationPlan> {
        if !self.can_redo() {
            return None;
        }
        let next = self.future.pop()?;
        let plan = RestorationPlan::between(&self.state, &next);
        self.past.push(std::mem::replace(&mut self.state, next));
        Some(plan)
    }
}
