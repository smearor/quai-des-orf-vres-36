//! `CornerHouse` - a flatiron house placed at a ring vertex plus
//! its edge frontage.

use crate::map::blocks::House;
use tangible_objects_model::Millimeters;

/// A flatiron house at a ring vertex plus the frontage it
/// consumes on the adjacent edges - the row-house pass starts
/// after it.
pub(super) struct CornerHouse {
    /// The house filling the wedge.
    pub house: House,
    /// Frontage consumed on the previous edge (mm).
    pub prev: Millimeters,
    /// Frontage consumed on the next edge (mm).
    pub next: Millimeters,
}
