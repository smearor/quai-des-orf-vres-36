//! `LinearityCause` - why a route graph fails the linearity
//! criterion (Q3/Q7).

use crate::StationCount;
use crate::StationDegree;
use crate::StationId;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

/// Why a route graph fails the linearity criterion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum LinearityCause {
    /// The line's stations are not one connected chain.
    #[error("line is not connected")]
    Disconnected,
    /// A station has more than two neighbours on the line (a branch).
    #[error("station {station} branches ({degree} neighbours)")]
    Branched {
        /// The station where the line forks.
        station: StationId,
        /// Its degree on the line.
        degree: StationDegree,
    },
    /// The line forms a ring - no distinct start and end stations.
    #[error("line is cyclic (no endpoints)")]
    Cyclic,
    /// The line has more than two endpoints (a tree, not a chain).
    #[error("line has {count} endpoints instead of 2")]
    EndpointCount {
        /// How many degree-1 stations the line has.
        count: StationCount,
    },
}
