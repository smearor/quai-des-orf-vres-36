//! Game session for Quai des Orfèvres 36.
//!
//! Session lifecycle ([`GameSession`]), configuration
//! ([`GameConfig`]/[`TangibleMode`]), the past/future undo stacks and
//! autosave persistence (`game.toml`). Move validation lives in
//! `qdo36-moves`; physical restoration steps in `qdo36-undo`.

pub mod game_config;
pub mod game_session;
pub mod game_state;
pub mod player_token;
pub mod reveal_preset;
pub mod session_error;
pub mod tangible_mode;

pub use game_config::GameConfig;
pub use game_session::GameSession;
pub use game_state::GameState;
pub use player_token::PlayerToken;
pub use reveal_preset::RevealPreset;
pub use session_error::SessionError;
pub use tangible_mode::TangibleMode;
