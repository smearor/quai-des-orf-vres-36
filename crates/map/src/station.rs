use crate::Neighbor;
use crate::StationId;
use crate::StationKind;
use crate::StationName;
use std::collections::BTreeSet;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// A node of the Paris game graph.
///
/// `pos` is the station's drawing position in map millimeters;
/// `neighbors` is materialized by [`crate::GameMap`] validation
/// for O(1) adjacency lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    /// Unique station number shown on the board.
    pub id: StationId,
    /// Display name (proper noun, not localized).
    pub name: StationName,
    /// Position in map millimeters.
    pub pos: Vec2<Millimeters>,
    /// Special roles of this station (start deal, batobus pier).
    pub kinds: BTreeSet<StationKind>,
    /// Adjacent stations with the transport mode to reach them.
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
}
