//! Station graph vocabulary: stations, their ids/kinds/names and
//! adjacency entries.

pub mod count;
pub mod id;
pub mod kind;
pub mod name;
pub mod neighbor;

pub use count::StationCount;
pub use id::StationId;
pub use kind::StationKind;
pub use name::StationName;
pub use neighbor::Neighbor;

use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;
use tangible_objects_model::serde_pair;

/// A node of the city game graph.
///
/// `pos` is the station's drawing position in map millimeters;
/// `neighbors` is materialized by [`crate::GameMap`] validation
/// for O(1) adjacency lookup - it is absent from the dataset TOML and
/// defaults to empty.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Station {
    /// Unique station number shown on the board.
    pub id: StationId,
    /// Display name (proper noun, not localized).
    pub name: StationName,
    /// Position in map millimeters; TOML shape `pos_mm = [x, y]`.
    #[serde(rename = "pos_mm", with = "serde_pair")]
    pub pos: Vec2<Millimeters>,
    /// Special roles of this station (start deal, batobus pier);
    /// empty means a plain station.
    #[serde(default)]
    pub kinds: BTreeSet<StationKind>,
    /// Adjacent stations with the transport mode to reach them.
    /// Derived during map validation; always `[]` in the dataset.
    #[serde(default)]
    pub neighbors: Vec<Neighbor>,
}

impl Station {
    /// Returns `true` when this station is eligible for the start deal.
    pub fn is_start(&self) -> bool {
        self.kinds.contains(&StationKind::Start)
    }

    /// Returns `true` when this station is a batobus pier.
    pub fn is_batobus(&self) -> bool {
        self.kinds.contains(&StationKind::Batobus)
    }

    /// Returns `true` when this station has an underground access.
    pub fn is_underground(&self) -> bool {
        self.kinds.contains(&StationKind::Underground)
    }
}
