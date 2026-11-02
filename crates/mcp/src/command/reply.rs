//! `CommandResult`/`CommandReply` - the typed answers the game host
//! sends back over the request's oneshot channel.

use qdo36_material::Role;
use qdo36_moves::Hop;
use qdo36_moves::MoveOutcome;
use qdo36_session::GameConfig;
use qdo36_session::PlayerToken;
use serde::Deserialize;
use serde::Serialize;

use crate::GameStateView;
use crate::McpError;
use crate::ScreenshotReply;

/// A successful command outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandResult {
    /// `NewGame` dealt and is waiting in the Deal phase.
    GameDealt,
    /// `Begin` opened the turn cycle.
    GameBegan,
    /// `SetConfig` applied; echoes the effective config.
    ConfigApplied(GameConfig),
    /// `Join` issued a token for the claimed role.
    Joined {
        /// The issued credential.
        token: PlayerToken,
        /// The seat the token is bound to.
        role: Role,
    },
    /// `GameState` view scoped to the token's role.
    State(Box<GameStateView>),
    /// `LegalMoves` - the hops the role may take now.
    Legal(Vec<Hop>),
    /// `Move` outcome (applied or game-ending).
    Moved(MoveOutcome),
    /// `Screenshot` PNG payload.
    Screenshot(ScreenshotReply),
}

/// The reply channel payload: a result or a typed failure.
pub type CommandReply = Result<CommandResult, McpError>;
