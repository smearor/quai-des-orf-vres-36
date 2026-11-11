//! `Angle` - a planar polygon angle (radians).

use smearor_commons_geometry::RotationRad;

/// A planar angle in radians - interior angles, arc sweeps and
/// direction deltas inside the block geometry. Alias of the
/// model crate's [`RotationRad`], so the shared radian
/// operations (`sin_cos`, `signed_delta_to`, `interior`, ...)
/// apply.
pub type Angle = RotationRad;
