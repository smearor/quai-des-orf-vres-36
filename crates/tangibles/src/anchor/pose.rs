use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;
use tangible_objects_model::RotationRad;
use tangible_objects_model::SmoothingAlpha;
use tangible_objects_model::TrackedTangible;
use tangible_objects_model::Vec2;

/// The Eiffel tower's measured pose on the surface.
///
/// `center` is the smoothed centroid in surface millimeters,
/// `rotation_rad` is correspondence-anchored to the profile's pin
/// frame (tangible-objects-rs pin-map fix), so it is stable and repeatable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnchorPose {
    /// Tangible centroid on the touch surface.
    pub center: Vec2<Millimeters>,
    /// Tangible orientation in radians (profile pin frame).
    pub rotation_rad: RotationRad,
}

impl AnchorPose {
    /// The neutral pose at the surface origin, facing north.
    pub const ZERO: Self = Self {
        center: Vec2::ZERO,
        rotation_rad: RotationRad::ZERO,
    };

    /// Interpolates toward `to` by `factor` (`0.0` = `self`, `1.0` =
    /// `to`): linear in position, shortest-arc in rotation.
    pub fn lerp(self, to: AnchorPose, factor: f64) -> AnchorPose {
        Self {
            center: Vec2::new(self.center.x + factor * (to.center.x - self.center.x), self.center.y + factor * (to.center.y - self.center.y)),
            rotation_rad: RotationRad::ema(self.rotation_rad, to.rotation_rad, SmoothingAlpha::new(factor)),
        }
    }
}

impl From<&TrackedTangible> for AnchorPose {
    fn from(tangible: &TrackedTangible) -> Self {
        Self {
            center: tangible.center,
            rotation_rad: tangible.rotation_rad,
        }
    }
}
