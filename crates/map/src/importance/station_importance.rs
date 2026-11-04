//! `StationImportance` - a station's score + tertile group.

use crate::importance::ImportanceClass;
use crate::importance::ImportanceScore;

/// Per-station importance data for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StationImportance {
    /// The weighted score.
    pub score: ImportanceScore,
    /// The tertile group the score falls into.
    pub class: ImportanceClass,
}
