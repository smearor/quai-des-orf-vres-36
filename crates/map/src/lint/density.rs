//! `TransportDensity` - per-layer density metrics (Q8: density
//! differs per transport - informational, not a finding).

use crate::EdgeCount;
use crate::StationCount;
use crate::lint::MeanDegree;
use serde::Deserialize;
use serde::Serialize;

/// How dense one transport layer is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransportDensity {
    /// Edges on this layer.
    pub edges: EdgeCount,
    /// Stations the layer touches.
    pub stations_served: StationCount,
    /// Mean edge-degree across served stations (2*edges/served).
    pub avg_degree: MeanDegree,
}
