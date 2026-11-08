//! `Finding` - one map-quality violation (`GameMap::lint`).
//!
//! The dataset quality criteria (Q1-Q9, WP10): every finding kind
//! maps to exactly one criterion.

use crate::LineName;
use crate::StationCount;
use crate::StationDegree;
use crate::StationId;
use crate::Transport;
use crate::lint::ComponentCount;
use crate::lint::EdgePair;
use crate::lint::LinearityCause;
use crate::lint::MissingService;
use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;
use thiserror::Error;

/// A dataset quality violation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum Finding {
    /// Two edges of the same layer cross without a shared station
    /// (Q1: each transport net is planar; metro lines may cross
    /// each other, only same-line self-crossings report).
    #[error("{transport} edges cross: {first:?} x {second:?}")]
    Crossing {
        /// The layer the crossing sits on.
        transport: Transport,
        /// First edge.
        first: EdgePair,
        /// Second edge.
        second: EdgePair,
    },
    /// A transport's adjacency graph is split into islands (Q2).
    #[error("{transport} net is disconnected ({components} components: {sizes:?} stations)")]
    Disconnected {
        /// The transport layer.
        transport: Transport,
        /// How many components the net has.
        components: ComponentCount,
        /// Stations per component, sorted descending.
        sizes: Vec<StationCount>,
    },
    /// A metro line is not a simple path (Q3).
    #[error("metro line {line} is not linear: {cause}")]
    LineNotLinear {
        /// The failing line.
        line: LineName,
        /// Why it fails.
        cause: LinearityCause,
    },
    /// The batobus pier chain is not a simple path (Q7).
    #[error("batobus route is not linear: {cause}")]
    BatobusNotLinear {
        /// Why it fails.
        cause: LinearityCause,
    },
    /// A metro line shares no station with any other line - no
    /// transfer possible (Q4).
    #[error("metro line {0} shares no station with any other line")]
    IsolatedLine(LineName),
    /// A metro station lacks taxi and/or bus service (Q5).
    #[error("metro station {station} lacks {missing:?} service")]
    MetroStationUnserved {
        /// The underserviced station.
        station: StationId,
        /// Which services are missing.
        missing: MissingService,
    },
    /// Fewer than 90% of bus stations also have taxi service (Q6).
    #[error("only {taxi_covered}/{bus_total} bus stations have taxi service (< 90%)")]
    BusCoverageBelow {
        /// Bus stations with taxi service.
        taxi_covered: StationCount,
        /// All bus stations.
        bus_total: StationCount,
    },
    /// A station has no edges at all - unreachable from anywhere.
    #[error("station {0} is orphaned (degree 0)")]
    OrphanedStation(StationId),
    /// A station has exactly one edge overall - a dead end that
    /// traps whoever enters (Q9, informational).
    #[error("station {0} is a leaf (degree 1)")]
    LeafStation(StationId),
    /// Two stations sit on the same position (Q9).
    #[error("stations {0} and {1} share the same position")]
    DuplicatePosition(StationId, StationId),
    /// A station lies almost on an edge it does not belong to - the
    /// route visually runs through the station without serving it,
    /// so the direct edge is hard to tell from the detour via the
    /// station (Q14, readability).
    #[error("{transport} edge {edge:?} runs within {distance_mm:.1} of station {station}")]
    StationOnEdge {
        /// The transport layer the offending edge belongs to.
        transport: Transport,
        /// The edge passing the station.
        edge: EdgePair,
        /// The station sitting on the line.
        station: StationId,
        /// Perpendicular distance in map millimeters.
        distance_mm: Millimeters,
    },
    /// A start-deal station has fewer than two edges - a dealt
    /// detective would start trapped (Q10).
    #[error("start station {station} is underconnected (degree {degree})")]
    UnderconnectedStart {
        /// The weak start station.
        station: StationId,
        /// Its total edge degree.
        degree: StationDegree,
    },
    /// The density order is inverted: a sparser-class layer carries
    /// more edges than a denser one, expected taxi >= bus >= metro
    /// (Q11, soft).
    #[error("density inversion: {sparser} has more edges than {denser}")]
    DensityInversion {
        /// The layer that should be sparser but is denser.
        sparser: Transport,
        /// The layer that should be denser.
        denser: Transport,
    },
}
