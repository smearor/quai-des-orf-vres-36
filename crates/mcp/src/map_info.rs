//! `MapInfo` - the loaded map's static facts for MCP consumers (WP05
//! data model).

use qdo36_map::CityName;
use qdo36_map::EdgeCount;
use qdo36_map::GameMap;
use qdo36_map::StationCount;
use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::Region;

/// Static facts about the loaded city map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapInfo {
    /// The city this dataset describes.
    pub name: CityName,
    /// Station count.
    pub stations: StationCount,
    /// Edge count (undirected adjacencies).
    pub edges: EdgeCount,
    /// Map coordinate bounds `[x_min, y_min, x_max, y_max]`.
    pub bounds_mm: Region,
    /// The Eiffel tower's anchor station position.
    pub anchor: Option<Vec2<Millimeters>>,
}

impl MapInfo {
    /// Captures the static facts of a loaded map; `anchor` is the
    /// configured anchor station's position.
    pub fn from_map(map: &GameMap, anchor: Option<Vec2<Millimeters>>) -> Self {
        Self {
            name: map.meta().name.clone(),
            stations: map.station_count(),
            edges: map.edge_count(),
            bounds_mm: map.bounds(),
            anchor,
        }
    }
}
