//! `StateSnapshot` - the shared watch document the app publishes and
//! the MCP tools serve.

use qdo36_session::GameSession;
use serde::Deserialize;
use serde::Serialize;

use crate::anchor_state_snapshot::AnchorStateSnapshot;
use crate::map_info::MapInfo;

/// The app's live state document - additive: new sections append,
/// never replace (WP05 R6).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Live anchor pose and detection.
    pub anchor: AnchorStateSnapshot,
    /// Static map facts, `Some` once a city map is loaded.
    pub map: Option<MapInfo>,
    /// The live session the app hosts; `None` until a game is dealt.
    /// Carries Fugitive's hidden position - the tools, never the
    /// transport, enforce the secrecy split (WP05 R4).
    pub game: Option<GameSession>,
}
