//! `CornerNose` - the flatiron's tangent nose arc plus the
//! facade data it consumes.

use crate::map::Polyline;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

/// The flatiron's street nose: the facade endpoints on both
/// adjacent edges, the tangent arc points between them, and the
/// frontage consumed on each edge.
pub(super) struct CornerNose {
    /// The facade endpoint on the previous edge (mm).
    pub front_prev: Vec2<Millimeters>,
    /// The facade endpoint on the next edge (mm).
    pub front_next: Vec2<Millimeters>,
    /// The nose arc between the two facade endpoints:
    /// `nose_prev`, the rounded sweep, `nose_next` (mm).
    pub points: Polyline,
    /// Frontage consumed on the previous edge (mm).
    pub facade_prev: Millimeters,
    /// Frontage consumed on the next edge (mm).
    pub facade_next: Millimeters,
}
