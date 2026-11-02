use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;

/// The touch surface's size in millimeters (from `deviceInfo`).
///
/// The bounds every field set is built against and clipped into, and
/// the scale basis for surface-mm ↔ pixel conversion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct SurfaceMm {
    /// Surface width in screen mm.
    pub width: Millimeters,
    /// Surface height in screen mm.
    pub height: Millimeters,
}

impl SurfaceMm {
    /// Whether `x` lies within the surface's horizontal bounds.
    pub fn contains_x(&self, x: Millimeters) -> bool {
        x >= Millimeters::default() && x <= self.width
    }
}

impl From<(f64, f64)> for SurfaceMm {
    fn from((width, height): (f64, f64)) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

impl From<(Millimeters, Millimeters)> for SurfaceMm {
    fn from((width, height): (Millimeters, Millimeters)) -> Self {
        Self { width, height }
    }
}

impl From<SurfaceMm> for (f64, f64) {
    fn from(surface: SurfaceMm) -> Self {
        (surface.width.as_f64(), surface.height.as_f64())
    }
}
