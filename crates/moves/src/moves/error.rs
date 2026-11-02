use crate::GamePhase;
use crate::MoveShape;
use qdo36_map::StationId;
use qdo36_map::Transport;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::TicketKind;
use thiserror::Error;

/// Errors a [`crate::MoveRequest`] can fail validation with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MoveError {
    /// The game is not in a phase that accepts moves.
    #[error("moves are not accepted in phase {actual} (expected {expected})")]
    WrongPhase {
        /// The phase that accepts moves.
        expected: GamePhase,
        /// The phase the game is actually in.
        actual: GamePhase,
    },
    /// A different player's turn is active.
    #[error("not the turn of {actual}; expected {expected}")]
    NotPlayersTurn {
        /// The role that may move now.
        expected: Role,
        /// The role that tried to move.
        actual: Role,
    },
    /// No map edge connects the two stations via this transport.
    #[error("no {transport} route between {from} and {to}")]
    NoRoute {
        /// Start station of the attempted hop.
        from: StationId,
        /// Attempted destination.
        to: StationId,
        /// Transport mode the route was attempted with.
        transport: Transport,
    },
    /// The player has no ticket of the requested kind left.
    #[error("{role} has no {ticket} tickets left")]
    TicketExhausted {
        /// The role that ran out of tickets.
        role: Role,
        /// The exhausted ticket kind.
        ticket: TicketKind,
    },
    /// The destination already holds another detective.
    #[error("station {0} is already occupied by pawn {1}")]
    DestinationOccupied(StationId, PawnId),
    /// The station does not exist on the map.
    #[error("unknown station {0}")]
    UnknownStation(StationId),
    /// This ticket kind is reserved for Fugitive.
    #[error("{0} tickets are Fugitive-only")]
    FugitiveOnlyTicket(TicketKind),
    /// A batobus route needs Fugitive's black ticket.
    #[error("batobus routes require the black ticket")]
    BatobusRequiresBlack,
    /// The move shape does not fit the ticket (e.g. a double ticket
    /// needs a two-hop move and vice versa).
    #[error("ticket {ticket} requires a {expected} move, got {actual}")]
    TicketShapeMismatch {
        /// The ticket presented with the wrong move shape.
        ticket: TicketKind,
        /// The shape the ticket requires.
        expected: MoveShape,
        /// The shape the request carries.
        actual: MoveShape,
    },
}
