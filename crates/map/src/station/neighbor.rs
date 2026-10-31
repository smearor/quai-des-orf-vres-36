use crate::StationId;
use crate::Transport;
use serde::Deserialize;
use serde::Serialize;

/// An adjacent station reachable from a station in one move.
///
/// Materialized adjacency of the game graph - one `Neighbor` per
/// direction of each [`crate::Edge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Neighbor {
    /// The adjacent station.
    pub station: StationId,
    /// Transport mode of the connecting route.
    pub transport: Transport,
}
