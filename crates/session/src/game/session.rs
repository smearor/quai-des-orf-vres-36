use crate::GameConfig;
use crate::GameState;
use serde::Deserialize;
use serde::Serialize;

/// One running (or finished) game with its undo history.
///
/// Mirrors tangibles-chess's `GameSession`: `past`/`future` stacks
/// implement full undo/redo - a move pushes the pre-move state onto
/// `past` and clears `future`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    /// Configuration fixed at setup.
    pub config: GameConfig,
    /// Current state.
    pub state: GameState,
    /// Undo stack - one entry per committed move.
    pub past: Vec<GameState>,
    /// Redo stack - repopulated by undo.
    pub future: Vec<GameState>,
}

impl GameSession {
    /// Whether undo is possible right now (also gated by
    /// `config.allow_undo` and `InProgress`/replay semantics).
    pub fn can_undo(&self) -> bool {
        self.config.allow_undo && !self.past.is_empty()
    }

    /// Whether redo is possible right now.
    pub fn can_redo(&self) -> bool {
        self.config.allow_undo && !self.future.is_empty()
    }
}
