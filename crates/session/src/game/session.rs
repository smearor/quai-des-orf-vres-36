//! `GameSession` - the rules engine: deal, move validation, turn
//! cycle, win conditions, undo/redo.
//!
//! A command-centric state machine: every input path (pawn, tap,
//! keypad, MCP) produces a [`MoveRequest`] and passes the same
//! validation pipeline - phase, turn, ticket, route, occupancy -
//! before [`GameSession::apply`] mutates state. `past`/`future` give
//! chess-style undo/redo with a physical [`RestorationPlan`].

use qdo36_map::GameMap;
use qdo36_map::StationId;
use qdo36_map::Transport;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_material::SeatCount;
use qdo36_material::TicketCounts;
use qdo36_material::TicketKind;
use qdo36_moves::GameLog;
use qdo36_moves::GameOutcome;
use qdo36_moves::GamePhase;
use qdo36_moves::Hop;
use qdo36_moves::MoveError;
use qdo36_moves::MoveOutcome;
use qdo36_moves::MoveRecord;
use qdo36_moves::MoveRequest;
use qdo36_moves::MoveShape;
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

    /// The actor's current station.
    fn position_of(&self, role: Role) -> Option<StationId> {
        match role {
            Role::Fugitive => Some(self.state.fugitive_pos),
            Role::Detective(pawn) => self.state.positions.get(&pawn).copied(),
        }
    }

    /// Whether a detective pawn already stands on `station`.
    fn station_free(&self, station: StationId) -> bool {
        !self.state.positions.values().any(|pos| *pos == station)
    }

    /// Whether `stock` pays `transport` for `role` - river/underground
    /// routes only exist for Fugitive's black ticket.
    fn can_afford(&self, stock: &TicketCounts, transport: Transport, role: Role) -> bool {
        match transport {
            Transport::Taxi => stock.taxi > 0,
            Transport::Bus => stock.bus > 0,
            Transport::Metro => stock.metro > 0,
            Transport::Batobus | Transport::Underground => matches!(role, Role::Fugitive) && stock.black > 0,
        }
    }

    fn validate_phase(&self) -> Result<(), MoveError> {
        if self.state.phase != GamePhase::InProgress {
            return Err(MoveError::WrongPhase {
                expected: GamePhase::InProgress,
                actual: self.state.phase,
            });
        }
        Ok(())
    }

    fn validate_turn(&self, actor: Role) -> Result<(), MoveError> {
        if self.state.active != actor {
            return Err(MoveError::NotPlayersTurn {
                expected: self.state.active,
                actual: actor,
            });
        }
        Ok(())
    }

    fn validate_ticket(&self, request: &MoveRequest) -> Result<(), MoveError> {
        if request.ticket.is_fugitive_only() && !matches!(request.actor, Role::Fugitive) {
            return Err(MoveError::FugitiveOnlyTicket(request.ticket));
        }
        let expected = match request.ticket {
            TicketKind::Double => MoveShape::Double,
            _ => MoveShape::Single,
        };
        if request.kind.shape() != expected {
            return Err(MoveError::TicketShapeMismatch {
                ticket: request.ticket,
                expected,
                actual: request.kind.shape(),
            });
        }
        let stock = self.state.tickets.get(&request.actor).copied().unwrap_or_default();
        if stock.get(request.ticket) == 0 {
            return Err(MoveError::TicketExhausted {
                role: request.actor,
                ticket: request.ticket,
            });
        }
        Ok(())
    }

    /// Every hop must start/end on real stations, ride an edge of a
    /// ticket-compatible transport, and land on an unoccupied station.
    fn validate_hops(&self, map: &GameMap, request: &MoveRequest) -> Result<(), MoveError> {
        let required = Transport::for_ticket(request.ticket);
        let mut passed = self.position_of(request.actor);
        for hop in request.kind.hops() {
            if map.station(hop.to).is_none() {
                return Err(MoveError::UnknownStation(hop.to));
            }
            if passed.is_some_and(|at| at != hop.from) {
                return Err(MoveError::NoRoute {
                    from: hop.from,
                    to: hop.to,
                    transport: hop.transport,
                });
            }
            if matches!(hop.transport, Transport::Batobus | Transport::Underground) && request.ticket != TicketKind::Black {
                return Err(MoveError::BatobusRequiresBlack);
            }
            if required.is_some_and(|transport| transport != hop.transport) {
                return Err(MoveError::NoRoute {
                    from: hop.from,
                    to: hop.to,
                    transport: hop.transport,
                });
            }
            if !map.neighbors_by(hop.from, hop.transport).any(|to| to == hop.to) {
                return Err(MoveError::NoRoute {
                    from: hop.from,
                    to: hop.to,
                    transport: hop.transport,
                });
            }
            if let Some((pawn, _)) = self.state.positions.iter().find(|(_, pos)| **pos == hop.to) {
                return Err(MoveError::DestinationOccupied(hop.to, *pawn));
            }
            passed = Some(hop.to);
        }
        Ok(())
    }

    /// Mutates state: snapshot for undo, position, ticket spend, the
    /// detective-ticket transfer to Fugitive, log entry, then win,
    /// stuck, turn-advance and round-cap checks in spec order.
    fn apply(&mut self, map: &GameMap, request: MoveRequest) -> MoveOutcome {
        let destination = request.kind.destination();
        self.past.push(self.state.clone());
        self.future.clear();
        let record = MoveRecord {
            round: self.state.round,
            actor: request.actor,
            ticket: request.ticket,
            kind: request.kind,
        };
        match request.actor {
            Role::Fugitive => self.state.fugitive_pos = destination,
            Role::Detective(pawn) => {
                self.state.positions.insert(pawn, destination);
            }
        }
        if let Some(stock) = self.state.tickets.get_mut(&request.actor) {
            stock.spend(request.ticket);
        }
        if matches!(request.actor, Role::Detective(_))
            && let Some(pool) = self.state.tickets.get_mut(&Role::Fugitive)
        {
            pool.gain(request.ticket);
        }
        self.state.log.entries.push(record);
        if let Role::Detective(_) = request.actor
            && destination == self.state.fugitive_pos
        {
            return self.finish(record, GameOutcome::DetectivesWin { caught_at: destination });
        }
        if self.detectives_stuck(map) {
            return self.finish(record, GameOutcome::DetectivesStuck);
        }
        self.advance_turn(map);
        if self.state.round > self.config.max_rounds {
            return self.finish(record, GameOutcome::FugitiveEscapes);
        }
        MoveOutcome::Applied(record)
    }

    /// Whether every detective is immobilized - Fugitive wins.
    fn detectives_stuck(&self, map: &GameMap) -> bool {
        !self.state.positions.is_empty()
            && self
                .state
                .positions
                .keys()
                .all(|seat| self.legal_destinations(map, Role::Detective(*seat)).is_empty())
    }

    /// Advances `active` in seat order (Fugitive → detectives in pawn
    /// order → next round with Fugitive), skipping stuck detectives.
    fn advance_turn(&mut self, map: &GameMap) {
        let mut next = match self.state.active {
            Role::Fugitive => self.state.positions.keys().min().map(|seat| Role::Detective(*seat)),
            Role::Detective(pawn) => self.state.positions.keys().find(|seat| **seat > pawn).map(|seat| Role::Detective(*seat)),
        };
        while let Some(Role::Detective(seat)) = next {
            if !self.legal_destinations(map, Role::Detective(seat)).is_empty() {
                break;
            }
            next = self.state.positions.keys().find(|after| **after > seat).map(|after| Role::Detective(*after));
        }
        match next {
            Some(role) => self.state.active = role,
            None => {
                self.state.round = RoundNumber::new(self.state.round.number().saturating_add(1));
                self.state.active = Role::Fugitive;
            }
        }
    }

    /// Ends the game: phase + outcome, wrapped with the move that
    /// caused it.
    fn finish(&mut self, record: MoveRecord, outcome: GameOutcome) -> MoveOutcome {
        self.state.phase = GamePhase::Finished;
        self.state.outcome = Some(outcome);
        MoveOutcome::Finished { record, outcome }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qdo36_map::Transport;
    use qdo36_moves::MoveKind;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// A dealt session on the bundled Paris map with a fixed seed
    /// (reproducible positions).
    fn dealt() -> (GameSession, GameMap) {
        let map = GameMap::bundled().unwrap_or_else(|_| panic!("bundled map must load"));
        let config = GameConfig::for_map(map.meta());
        let mut rng = StdRng::seed_from_u64(36);
        let session = GameSession::deal(config, &map, &mut rng).unwrap_or_else(|err| panic!("deal must succeed: {err}"));
        (session, map)
    }

    /// A session past the Deal phase, Fugitive to move.
    fn running() -> (GameSession, GameMap) {
        let (mut session, map) = dealt();
        session.begin().unwrap_or_else(|_| panic!("begin must succeed"));
        (session, map)
    }

    /// A single-hop request for the active role along a given hop.
    fn request(session: &GameSession, hop: Hop) -> MoveRequest {
        let ticket = match hop.transport {
            Transport::Taxi => TicketKind::Taxi,
            Transport::Bus => TicketKind::Bus,
            Transport::Metro => TicketKind::Metro,
            Transport::Batobus | Transport::Underground => TicketKind::Black,
        };
        MoveRequest {
            actor: session.state.active,
            ticket,
            kind: MoveKind::Single(hop),
        }
    }

    #[test]
    fn deal_assigns_seats_tickets_and_hidden_fugitive() {
        let (session, map) = dealt();
        let seats = session.config.detectives.count() as usize;
        assert_eq!(session.state.positions.len(), seats, "every detective is seated");
        let mut distinct = session.state.positions.values().copied().collect::<Vec<_>>();
        distinct.push(session.state.fugitive_pos);
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), seats + 1, "every dealt station is distinct");
        for station in &distinct {
            assert!(map.start_stations().any(|s| s.id == *station), "{station} is a start station");
        }
        assert_eq!(session.state.phase, GamePhase::Deal);
        assert_eq!(session.state.tickets[&Role::Detective(PawnId::new(0))], session.config.detective_stock);
        assert_eq!(session.state.tickets[&Role::Fugitive].black, session.config.fugitive_stock.black);
    }

    #[test]
    fn deal_honors_configured_stocks() {
        let map = GameMap::bundled().unwrap_or_else(|_| panic!());
        let mut config = GameConfig::for_map(map.meta());
        config.detective_stock.taxi = 3;
        config.fugitive_stock.black = 1;
        let session = GameSession::deal(config, &map, &mut StdRng::seed_from_u64(36)).unwrap_or_else(|_| panic!());
        assert_eq!(session.state.tickets[&Role::Detective(PawnId::new(0))].taxi, 3);
        assert_eq!(session.state.tickets[&Role::Fugitive].black, 1);
    }

    #[test]
    fn deterministic_seed_repeats_the_deal() {
        let map = GameMap::bundled().unwrap_or_else(|_| panic!());
        let config = GameConfig::for_map(map.meta());
        let a = GameSession::deal(config.clone(), &map, &mut StdRng::seed_from_u64(7)).unwrap_or_else(|_| panic!());
        let b = GameSession::deal(config, &map, &mut StdRng::seed_from_u64(7)).unwrap_or_else(|_| panic!());
        assert_eq!(a.state.positions, b.state.positions);
        assert_eq!(a.state.fugitive_pos, b.state.fugitive_pos);
    }

    #[test]
    fn moves_are_rejected_during_deal() {
        let (mut session, map) = dealt();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        let err = session.request_move(&map, &request(&session, hop)).err().unwrap_or_else(|| panic!("must fail"));
        assert!(matches!(err, MoveError::WrongPhase { .. }));
    }

    #[test]
    fn wrong_actor_is_rejected() {
        let (mut session, map) = running();
        let other = Role::Detective(PawnId::new(0));
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        let request = MoveRequest {
            actor: other,
            ticket: TicketKind::Taxi,
            kind: MoveKind::Single(hop),
        };
        let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
        assert_eq!(
            err,
            MoveError::NotPlayersTurn {
                expected: Role::Fugitive,
                actual: other
            }
        );
    }

    #[test]
    fn detectives_cannot_use_fugitive_only_tickets() {
        let (mut session, map) = running();
        // Fugitive moves first so a detective becomes active.
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        let pawn = match session.state.active {
            Role::Detective(pawn) => pawn,
            Role::Fugitive => panic!("a detective must be active"),
        };
        let hop = session.legal_destinations(&map, Role::Detective(pawn))[0];
        let request = MoveRequest {
            actor: Role::Detective(pawn),
            ticket: TicketKind::Black,
            kind: MoveKind::Single(hop),
        };
        let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
        assert_eq!(err, MoveError::FugitiveOnlyTicket(TicketKind::Black));
    }

    #[test]
    fn double_ticket_requires_the_double_shape() {
        let (mut session, map) = running();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        let request = MoveRequest {
            actor: Role::Fugitive,
            ticket: TicketKind::Double,
            kind: MoveKind::Single(hop),
        };
        let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
        assert!(matches!(
            err,
            MoveError::TicketShapeMismatch {
                ticket: TicketKind::Double,
                ..
            }
        ));
    }

    #[test]
    fn batobus_demands_the_black_ticket() {
        let (mut session, map) = running();
        let pos = session.state.fugitive_pos;
        let river = map
            .neighbors(pos)
            .iter()
            .find(|n| matches!(n.transport, Transport::Batobus | Transport::Underground));
        let Some(river) = river else { return };
        let request = MoveRequest {
            actor: Role::Fugitive,
            ticket: TicketKind::Taxi,
            kind: MoveKind::Single(Hop {
                from: pos,
                to: river.station,
                transport: river.transport,
            }),
        };
        let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
        assert_eq!(err, MoveError::BatobusRequiresBlack);
    }

    #[test]
    fn applied_moves_update_state_log_and_turn() {
        let (mut session, map) = running();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        let outcome = session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        let record = match outcome {
            MoveOutcome::Applied(record) => record,
            MoveOutcome::Finished { .. } => panic!("one move cannot finish the game"),
        };
        assert_eq!(session.state.fugitive_pos, hop.to);
        assert_eq!(session.state.log.entries, vec![record]);
        assert!(matches!(session.state.active, Role::Detective(_)), "Fugitive hands over to detectives");
        assert!(session.can_undo());
    }

    #[test]
    fn occupied_destinations_are_rejected() {
        let (mut session, map) = running();
        // Find a detective station adjacent to Fugitive's position.
        let Some(hop) = map
            .neighbors(session.state.fugitive_pos)
            .iter()
            .find(|n| session.state.positions.values().any(|pos| *pos == n.station))
            .map(|n| Hop {
                from: session.state.fugitive_pos,
                to: n.station,
                transport: n.transport,
            })
        else {
            return;
        };
        let request = MoveRequest {
            actor: Role::Fugitive,
            ticket: TicketKind::Black,
            kind: MoveKind::Single(hop),
        };
        let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
        assert!(matches!(err, MoveError::DestinationOccupied(..)));
    }

    #[test]
    fn undo_restores_state_and_returns_a_plan() {
        let (mut session, map) = running();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        let detective = session.state.active;
        let hop = session.legal_destinations(&map, detective)[0];
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        let plan = session.undo().unwrap_or_else(|| panic!("undo must work"));
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(session.state.log.entries.len(), 1);
        assert!(session.can_redo());
        let plan = session.redo().unwrap_or_else(|| panic!("redo must work"));
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(session.state.log.entries.len(), 2);
    }

    #[test]
    fn undoing_a_fugitive_move_yields_an_empty_plan() {
        let (mut session, map) = running();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        let plan = session.undo().unwrap_or_else(|| panic!("undo must work"));
        assert!(plan.steps.is_empty(), "fugitive has no physical pawn");
    }

    #[test]
    fn landing_on_fugitive_wins_for_the_detectives() {
        let (mut session, map) = running();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        // Teleport a detective next to Fugitive for the scripted catch.
        let fugitive = session.state.fugitive_pos;
        let pawn = match session.state.active {
            Role::Detective(pawn) => pawn,
            Role::Fugitive => panic!(),
        };
        let adjacent = map
            .neighbors(fugitive)
            .iter()
            .find(|n| session.station_free(n.station))
            .unwrap_or_else(|| panic!("adjacent station"));
        session.state.positions.insert(pawn, adjacent.station);
        let request = MoveRequest {
            actor: Role::Detective(pawn),
            ticket: match adjacent.transport {
                Transport::Taxi => TicketKind::Taxi,
                Transport::Bus => TicketKind::Bus,
                Transport::Metro => TicketKind::Metro,
                Transport::Batobus | Transport::Underground => TicketKind::Black,
            },
            kind: MoveKind::Single(Hop {
                from: adjacent.station,
                to: fugitive,
                transport: adjacent.transport,
            }),
        };
        let outcome = session.request_move(&map, &request).unwrap_or_else(|err| panic!("{err}"));
        match outcome {
            MoveOutcome::Finished { outcome, .. } => {
                assert!(matches!(outcome, GameOutcome::DetectivesWin { caught_at } if caught_at == fugitive));
            }
            MoveOutcome::Applied(_) => panic!("the capture must end the game"),
        }
        assert_eq!(session.state.phase, GamePhase::Finished);
    }

    #[test]
    fn detective_tickets_feed_the_fugitive_pool() {
        let (mut session, map) = running();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        let before = session.state.tickets[&Role::Fugitive].taxi;
        let hop = session
            .legal_destinations(&map, session.state.active)
            .iter()
            .find(|h| h.transport == Transport::Taxi)
            .copied();
        let Some(hop) = hop else { return };
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            session.state.tickets[&Role::Fugitive].taxi,
            before.saturating_add(1),
            "the spent taxi lands in Fugitive's pool"
        );
    }

    #[test]
    fn public_view_hides_fugitive_until_reveal() {
        let (mut session, map) = running();
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
        let schedule = session.config.reveal_schedule();
        let view = session.state.log.fugitive_public_view(&schedule);
        assert_eq!(view[0].destination.is_none(), !schedule.is_reveal(session.state.round));
    }
}
