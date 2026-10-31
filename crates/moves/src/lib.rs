//! Move domain for Quai des Orfèvres 36.
//!
//! Move requests and records ([`MoveRequest`] → [`MoveRecord`]), the
//! game log with its public projection ([`GameLog::corbeau_public_view`]
//! - Corbeau secrecy), game phases, outcomes and validation errors.

pub mod game_log;
pub mod game_outcome;
pub mod game_phase;
pub mod hop;
pub mod move_error;
pub mod move_kind;
pub mod move_record;
pub mod move_request;
pub mod public_entry;
pub mod reveal_schedule;

pub use game_log::GameLog;
pub use game_outcome::GameOutcome;
pub use game_phase::GamePhase;
pub use hop::Hop;
pub use move_error::MoveError;
pub use move_kind::MoveKind;
pub use move_record::MoveRecord;
pub use move_request::MoveRequest;
pub use public_entry::PublicEntry;
pub use reveal_schedule::RevealSchedule;
