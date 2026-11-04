//! `EdgePair` - one edge's endpoint-pair for crossing reports.

use crate::LineName;
use crate::StationId;
use serde::Deserialize;
use serde::Serialize;

/// One edge endpoint-pair, for crossing reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgePair {
    /// Canonical lower endpoint.
    pub from: StationId,
    /// Canonical higher endpoint.
    pub to: StationId,
    /// The metro line, when the edge belongs to one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<LineName>,
}
