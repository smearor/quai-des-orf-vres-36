//! `NearestStation` - the station nearest to a queried position.

use crate::StationId;
use tangible_objects_model::Millimeters;

/// The station nearest to a queried map position, with its
/// distance - the [`crate::GameMap::nearest_station`] result (tap
/// hit-testing, pawn snap queries).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearestStation {
    /// The nearest station's id.
    pub station: StationId,
    /// Distance to the queried position in map millimeters.
    pub distance: Millimeters,
}
