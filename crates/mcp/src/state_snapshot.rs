//! `StateSnapshot` - the shared watch document the app publishes and
//! the MCP tools serve.

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
}
