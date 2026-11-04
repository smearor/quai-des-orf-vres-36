use crate::LineName;
use crate::StationId;
use crate::Transport;
use thiserror::Error;

/// Errors produced while loading or validating the map dataset.
#[derive(Debug, Error)]
pub enum MapError {
    /// The TOML dataset could not be parsed.
    #[error("map dataset is not valid TOML: {0}")]
    Parse(#[from] toml::de::Error),
    /// Two stations share the same id.
    #[error("duplicate station id {0}")]
    DuplicateStationId(StationId),
    /// An edge references a station that does not exist.
    #[error("edge references unknown station {0}")]
    MissingStation(StationId),
    /// An edge loops back onto the same station.
    #[error("self-edge on station {0}")]
    SelfEdge(StationId),
    /// The same route exists twice.
    #[error("duplicate {transport} edge between {from} and {to}")]
    DuplicateEdge {
        /// Canonical lower endpoint.
        from: StationId,
        /// Canonical higher endpoint.
        to: StationId,
        /// Transport mode of the duplicated route.
        transport: Transport,
    },
    /// A batobus edge terminates at a station that is no pier.
    #[error("batobus edge between {0} and {1} requires both ends to be batobus piers")]
    BatobusOnLand(StationId, StationId),
    /// An underground edge terminates at a station with no access.
    #[error("underground edge between {0} and {1} requires both ends to be underground accesses")]
    UndergroundOnSurface(StationId, StationId),
    /// A batobus pier is not flagged as a bridge station.
    #[error("batobus pier {0} must also be a bridge (the boat docks at bridge landings)")]
    BatobusOffBridge(StationId),
    /// A taxi/bus edge crosses a water polyline with no bridge on
    /// either end (roads cross water only via bridges).
    #[error("{transport} edge between {from} and {to} crosses water but neither end is a bridge")]
    RoadCrossesWater {
        /// One endpoint of the offending edge.
        from: StationId,
        /// The other endpoint.
        to: StationId,
        /// Transport mode of the crossing edge (`taxi` or `bus`).
        transport: Transport,
    },
    /// A station position lies outside the declared map bounds.
    #[error("station {0} lies outside the map bounds")]
    OutOfBounds(StationId),
    /// The declared map bounds are degenerate.
    #[error("map bounds are empty, inverted or non-finite")]
    InvalidBounds,
    /// The dataset contains no stations.
    #[error("map dataset has no stations")]
    EmptyMap,
    /// An edge references a line not declared in `[map.lines]`.
    #[error("edge references unknown line {0}")]
    UnknownLine(LineName),
    /// A non-metro edge carries a line (lines belong to metro routes).
    #[error("edge between {from} and {to} carries line {line} but is not a metro route")]
    LineOnNonMetro {
        /// One endpoint of the offending edge.
        from: StationId,
        /// The other endpoint.
        to: StationId,
        /// The misplaced line.
        line: LineName,
    },
}
