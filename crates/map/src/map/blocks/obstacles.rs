//! `Obstacles` - polygons a block's houses must stay out of.

use std::ops::Deref;

use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Polygon;

/// The obstacle polygons carving a block's free area: tunnel
/// corridors, decorative street corridors, park blobs, landmark
/// footprints - houses intersecting them are skipped.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Obstacles(pub Vec<Polygon<Millimeters>>);

impl<'a> IntoIterator for &'a Obstacles {
    type Item = &'a Polygon<Millimeters>;
    type IntoIter = std::slice::Iter<'a, Polygon<Millimeters>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Obstacles {
    type Target = [Polygon<Millimeters>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Polygon<Millimeters>>> for Obstacles {
    fn from(polygons: Vec<Polygon<Millimeters>>) -> Self {
        Self(polygons)
    }
}
