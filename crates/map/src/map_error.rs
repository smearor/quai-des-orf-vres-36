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
    /// A station position lies outside the declared map bounds.
    #[error("station {0} lies outside the map bounds")]
    OutOfBounds(StationId),
    /// The declared map bounds are degenerate.
    #[error("map bounds are empty, inverted or non-finite")]
    InvalidBounds,
    /// The dataset contains no stations.
    #[error("map dataset has no stations")]
    EmptyMap,
}
