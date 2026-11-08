//! `ParkSpot` - a park blob inside a city block.

use qdo36_map::ParkName;
use qdo36_map::Polygon;

/// A dataset park grown into a block: the organic blob polygon
/// the renderer fills green and names, kept free of houses.
#[derive(Debug, Clone, PartialEq)]
pub struct ParkSpot {
    /// Park name (dataset).
    pub name: ParkName,
    /// Blob polygon in map mm.
    pub polygon: Polygon,
}
