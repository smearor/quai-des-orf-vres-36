//! Applying a validated move to the state: the memento push, the
//! position/stock/log updates, and the follow-up checks (win, stuck,
//! turn advance, round cap).

use qdo36_map::GameMap;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_moves::GameOutcome;
use qdo36_moves::GamePhase;
use qdo36_moves::MoveOutcome;
use qdo36_moves::MoveRecord;
use qdo36_moves::MoveRequest;

use crate::game::session::GameSession;

impl GameSession {
    /// Mutates state: snapshot for undo, position, ticket spend, the
    /// detective-ticket transfer to Fugitive, log entry, then win,
    /// stuck, turn-advance and round-cap checks in spec order.
    pub(super) fn apply(&mut self, map: &GameMap, request: MoveRequest) -> MoveOutcome {
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
