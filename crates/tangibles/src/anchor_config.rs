use crate::MapScale;
use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;
use tangible_objects_model::RotationRad;
use tangible_objects_model::Vec2;

/// Calibration of the map anchor: which map point the tower pins down,
/// how the map is oriented, and the resting zoom.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnchorConfig {
    /// The map position the tower stands on - Champ de Mars, i.e. the
    /// Eiffel tower station. The map is translated so this point lands
    /// under the tangible's centroid.
    pub anchor_map_pos: Vec2<Millimeters>,
    /// Offset between the profile's pin frame and "map north", in
    /// degrees. Calibrated once per physical tower build (the profile
    /// has no canonical "forward").
    pub heading_offset: RotationRad,
    /// Resting zoom; the overlay's zoom controls move from here.
    pub base_scale: MapScale,
    /// Map coordinate bounds (from the dataset, kept for the transform).
    ///
    /// The bounds deliberately extend beyond the station envelope:
    /// stations sit in a relevant inner area while the drawable
    /// surroundings (river, streets, backdrop) fill the margin, so
    /// panning the viewport never leaves the city.
    pub map_bounds: Region,
}

impl AnchorConfig {
    /// The anchor point's map position equals the Eiffel tower station.
    ///
    /// `tower_station` is looked up from the map (station 1 in
    /// `paris.toml`).
    pub fn at_station(map_bounds: Region, tower_station: Vec2<Millimeters>) -> Self {
        Self {
            anchor_map_pos: tower_station,
            heading_offset: RotationRad::new(0.0),
            base_scale: MapScale::default(),
            map_bounds,
        }
    }
}
