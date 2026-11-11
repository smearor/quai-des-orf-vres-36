//! `CornerFootprint` - a flatiron footprint plus its edge
//! frontage.

use crate::map::Polygon;
use smearor_commons_geometry::Millimeters;

/// A flatiron footprint at a ring vertex plus the frontage it
/// consumes on the adjacent edges - the row-house pass starts
/// after it.
pub(super) struct CornerFootprint {
    /// The footprint polygon in map mm.
    pub outline: Polygon,
    /// Frontage consumed on the previous edge (mm).
    pub prev: Millimeters,
    /// Frontage consumed on the next edge (mm).
    pub next: Millimeters,
}
