//! `Obstacles` - polygons a block's houses must stay out of.

use std::ops::Deref;

use crate::map::Polygon;

/// The obstacle polygons carving a block's free area: tunnel
/// corridors, decorative street corridors, park blobs, landmark
/// footprints - houses intersecting them are skipped.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Obstacles(pub Vec<Polygon>);

impl<'a> IntoIterator for &'a Obstacles {
    type Item = &'a Polygon;
    type IntoIter = std::slice::Iter<'a, Polygon>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Obstacles {
    type Target = [Polygon];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Polygon>> for Obstacles {
    fn from(polygons: Vec<Polygon>) -> Self {
        Self(polygons)
    }
}
