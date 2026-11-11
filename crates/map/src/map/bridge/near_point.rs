//! `NearPoint` - the nearest point on any water course.

use smearor_commons_geometry::Millimeters;

/// The nearest point on any water course to a candidate position:
/// its distance and where it lands (course, segment, fraction).
pub(super) struct NearPoint {
    /// Distance from the position to the projected point.
    pub distance: Millimeters,
    /// Index of the course the nearest point lies on.
    pub course: usize,
    /// Index of the course segment the point projects onto.
    pub segment: usize,
    /// Fraction along that segment (0 = start, 1 = end).
    pub fraction: f64,
}
