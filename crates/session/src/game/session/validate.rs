//! The validation pipeline: phase, turn, ticket, route, occupancy -
//! every check a `MoveRequest` must pass before `apply` may run.

use qdo36_map::GameMap;
use qdo36_map::StationId;
use qdo36_map::Transport;
use qdo36_material::Role;
use qdo36_material::TicketCounts;
use qdo36_material::TicketKind;
use qdo36_moves::GamePhase;
use qdo36_moves::MoveError;
use qdo36_moves::MoveRequest;
use qdo36_moves::MoveShape;

use crate::game::session::GameSession;

impl GameSession {
    /// The actor's current station.
    pub(super) fn position_of(&self, role: Role) -> Option<StationId> {
        match role {
            Role::Fugitive => Some(self.state.fugitive_pos),
            Role::Detective(pawn) => self.state.positions.get(&pawn).copied(),
        }
    }

    /// Whether a detective pawn already stands on `station`.
    pub(super) fn station_free(&self, station: StationId) -> bool {
        !self.state.positions.values().any(|pos| *pos == station)
    }

    /// Whether `stock` pays `transport` for `role` - river/underground
    /// routes only exist for Fugitive's black ticket.
    pub(super) fn can_afford(&self, stock: &TicketCounts, transport: Transport, role: Role) -> bool {
        match transport {
            Transport::Taxi => stock.taxi > 0,
            Transport::Bus => stock.bus > 0,
            Transport::Metro => stock.metro > 0,
            Transport::Batobus | Transport::Underground => matches!(role, Role::Fugitive) && stock.black > 0,
        }
    }

    pub(super) fn validate_phase(&self) -> Result<(), MoveError> {
        if self.state.phase != GamePhase::InProgress {
            return Err(MoveError::WrongPhase {
                expected: GamePhase::InProgress,
                actual: self.state.phase,
            });
        }
        Ok(())
    }

    pub(super) fn validate_turn(&self, actor: Role) -> Result<(), MoveError> {
        if self.state.active != actor {
            return Err(MoveError::NotPlayersTurn {
                expected: self.state.active,
                actual: actor,
            });
        }
        Ok(())
    }

    pub(super) fn validate_ticket(&self, request: &MoveRequest) -> Result<(), MoveError> {
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
    pub(super) fn validate_hops(&self, map: &GameMap, request: &MoveRequest) -> Result<(), MoveError> {
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
}
