//! `qdo36-session` - game/session lifecycle of Quai des Orfèvres 36.
//!
//! [`GameConfig`] (setup choices incl. [`TangibleMode`] and
//! [`RevealPreset`]), [`GameState`] snapshots, [`GameSession`] with
//! undo/redo stacks, [`PlayerToken`] MCP seats and [`SessionError`].

pub mod config;
pub mod error;
pub mod game;
pub mod player;

pub use config::GameConfig;
pub use config::RevealPreset;
pub use config::TangibleMode;
pub use error::SessionError;
pub use game::GameSession;
pub use game::GameState;
pub use player::PlayerToken;
