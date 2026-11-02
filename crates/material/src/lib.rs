//! Material vocabulary for Quai des Orfèvres 36.
//!
//! The "pieces" of the game: who sits at the table ([`Role`]), the
//! detective pawns ([`PawnId`], [`PawnColor`]) and the tickets they
//! spend ([`TicketKind`], [`TicketCounts`]). Leaf crate - depends on
//! serde only.

pub mod detective_count;
pub mod pawn_color;
pub mod pawn_id;
pub mod role;
pub mod round_number;
pub mod seat_count;
pub mod ticket_counts;
pub mod ticket_kind;

pub use detective_count::DetectiveCount;
pub use detective_count::DetectiveCountError;
pub use pawn_color::PawnColor;
pub use pawn_id::PawnId;
pub use role::Role;
pub use round_number::RoundNumber;
pub use seat_count::SeatCount;
pub use ticket_counts::TicketCounts;
pub use ticket_kind::TicketKind;
pub use ticket_kind::UnknownTicketKind;
