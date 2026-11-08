//! `DistrictLabel` - the quarter name a block carries.

use std::rc::Rc;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// The district name a block shows at its visual center: an
/// organizational unit the players can use to talk about map
/// areas ("Der Corbeau war in Saint-Germain").
#[derive(Debug, Clone)]
pub struct DistrictLabel {
    /// The quarter name.
    pub name: Rc<str>,
    /// Label center (map mm) - the deepest inside point of the
    /// inset outline.
    pub pos: Vec2<Millimeters>,
}
