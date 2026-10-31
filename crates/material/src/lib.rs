//! Material vocabulary for Quai des Orfèvres 36.
//!
//! The "pieces" of the game: who sits at the table ([`Role`], the
//! detective pawns ([`PawnId`]) and the tickets they spend. Leaf crate
//! - depends on serde only.

pub mod pawn_id;
pub mod role;

pub use pawn_id::PawnId;
pub use role::Role;
