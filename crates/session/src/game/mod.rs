//! Game lifecycle: state snapshots and the session with undo/redo
//! stacks.

pub mod session;
pub mod state;

pub use session::GameSession;
pub use state::GameState;
