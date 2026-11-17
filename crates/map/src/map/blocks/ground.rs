//! `Ground` - the city ground mask's polygons.

use std::ops::Deref;

use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Polygon;

/// The ground mask covering the city: every inner face's outline
/// expanded by each edge's setback - streets, corridors and
/// suppressed faces all rest on ground, never on texture.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ground(pub Vec<Polygon<Millimeters>>);

impl<'a> IntoIterator for &'a Ground {
    type Item = &'a Polygon<Millimeters>;
    type IntoIter = std::slice::Iter<'a, Polygon<Millimeters>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Ground {
    type Target = [Polygon<Millimeters>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Polygon<Millimeters>>> for Ground {
    fn from(polygons: Vec<Polygon<Millimeters>>) -> Self {
        Self(polygons)
    }
}
