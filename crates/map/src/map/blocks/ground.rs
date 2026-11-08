//! `Ground` - the city ground mask's polygons.

use std::ops::Deref;

use crate::map::Polygon;

/// The ground mask covering the city: every inner face's outline
/// expanded by each edge's setback - streets, corridors and
/// suppressed faces all rest on ground, never on texture.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ground(pub Vec<Polygon>);

impl<'a> IntoIterator for &'a Ground {
    type Item = &'a Polygon;
    type IntoIter = std::slice::Iter<'a, Polygon>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Ground {
    type Target = [Polygon];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Polygon>> for Ground {
    fn from(polygons: Vec<Polygon>) -> Self {
        Self(polygons)
    }
}
