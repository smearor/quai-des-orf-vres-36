use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;
use tangible_objects_model::RotationRad;
use tangible_objects_model::Vec2;

/// The Eiffel tower's measured pose on the surface.
///
/// `center` is the smoothed centroid in surface millimeters,
/// `rotation_rad` is correspondence-anchored to the profile's pin
/// frame (robofigure-rs pin-map fix), so it is stable and repeatable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnchorPose {
    /// Tangible centroid on the touch surface.
    pub center: Vec2<Millimeters>,
    /// Tangible orientation in radians (profile pin frame).
    pub rotation_rad: RotationRad,
}
