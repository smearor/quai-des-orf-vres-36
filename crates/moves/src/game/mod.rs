//! The game domain: lifecycle phase, outcome, the move log with its
//! public (Fugitive-redacted) projection and the reveal schedule.

pub mod log;
pub mod outcome;
pub mod phase;
pub mod public_entry;
pub mod reveal_schedule;

pub use log::GameLog;
pub use outcome::GameOutcome;
pub use phase::GamePhase;
pub use public_entry::PublicEntry;
pub use reveal_schedule::RevealSchedule;
