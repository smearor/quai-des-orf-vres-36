use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::SmoothingAlpha;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::TrackedTangible;

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

    /// Interpolates toward `to` by `alpha` (`0.0` = `self`, `1.0` =
    /// `to`): linear in position, shortest-arc in rotation.
    pub fn lerp(self, to: AnchorPose, alpha: SmoothingAlpha) -> AnchorPose {
        Self {
            center: self.center + (to.center - self.center) * alpha,
            rotation_rad: RotationRad::ema(self.rotation_rad, to.rotation_rad, alpha),
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
