//! `CommandKind` - the write commands crossing from MCP tools into
//! the app's session owner.

use qdo36_material::RoundNumber;
use qdo36_moves::MoveRequest;
use qdo36_session::GameConfig;
use qdo36_session::PlayerToken;
use qdo36_session::RevealPreset;
use serde::Deserialize;
use serde::Serialize;

use crate::RoleClaim;

/// One command for the game host.
///
/// The tool never mutates session state itself - it wraps the command
/// into a [`crate::GameRequest`] whose oneshot carries the reply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    /// Deals a fresh game (`phase: Deal → InProgress` once
    /// [`Self::Begin`] arrives). `None` uses the loaded map's
    /// defaults (`GameConfig::for_map`); `Some` deals with the given
    /// setup - needed for deal-shaping fields like detective count,
    /// ticket stocks or `allow_undo`, which `SetConfig` cannot change
    /// post-deal.
    NewGame {
        /// Optional explicit setup; `None` = map defaults.
        config: Option<GameConfig>,
    },
    /// Adjusts reveal schedule and max rounds pre-game (Deal phase
    /// only).
    SetConfig {
        /// Reveal preset override; `None` keeps the current choice.
        reveal: Option<RevealPreset>,
        /// Game length override; `None` keeps the current value.
        max_rounds: Option<RoundNumber>,
    },
    /// Leaves the Deal phase - moves are accepted afterwards.
    Begin,
    /// Seats an agent under a role claim; issues a `PlayerToken`.
    Join {
        /// Which seat the agent claims.
        claim: RoleClaim,
    },
    /// The role-scoped state for a token.
    GameState {
        /// The player's token.
        token: PlayerToken,
    },
    /// Legal destinations for a token's role.
    LegalMoves {
        /// The player's token.
        token: PlayerToken,
    },
    /// A move for a token's role (the host enforces `actor = role`).
    Move {
        /// The player's token.
        token: PlayerToken,
        /// The requested move; `request.actor` is overridden with the
        /// token's role.
        request: MoveRequest,
    },
    /// A PNG screenshot of the app window. Handled by the app-side
    /// drain, not the game host - the session knows no window.
    Screenshot,
}
