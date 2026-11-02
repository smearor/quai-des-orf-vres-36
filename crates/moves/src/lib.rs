//! `qdo36-moves` - the move domain of Quai des Orfèvres 36.
//!
//! Move vocabulary ([`Hop`], [`MoveKind`], [`MoveRequest`],
//! [`MoveRecord`], [`MoveError`]) plus game lifecycle types
//! ([`GamePhase`], [`GameOutcome`], [`GameLog`], [`PublicEntry`],
//! [`RevealSchedule`]). Pure domain, no session orchestration.

pub mod game;
pub mod moves;

pub use game::GameLog;
pub use game::GameOutcome;
pub use game::GamePhase;
pub use game::PublicEntry;
pub use game::RevealSchedule;
pub use moves::Hop;
pub use moves::MoveError;
pub use moves::MoveKind;
pub use moves::MoveOutcome;
pub use moves::MoveRecord;
pub use moves::MoveRequest;
pub use moves::MoveShape;
