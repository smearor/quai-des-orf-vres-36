//! `Districts` - the map's named city quarters.

use crate::map::District;
use crate::map::DistrictName;
use std::ops::Deref;
use std::ops::DerefMut;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// The map's named city quarters. Derefs to the `[District]`
/// slice.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Districts(pub Vec<District>);

impl Districts {
    /// The positioned quarters as `(position, name)` pairs - the
    /// names the inner blocks pin geographically.
    pub fn positioned(&self) -> impl Iterator<Item = (Vec2<Millimeters>, &DistrictName)> {
        self.0.iter().filter_map(|district| district.pos.map(|pos| (pos, &district.name)))
    }

    /// The names of quarters without a dataset position - the
    /// still-unnamed blocks fill them in dataset order.
    pub fn unpositioned_names(&self) -> impl Iterator<Item = DistrictName> + '_ {
        self.0.iter().filter(|district| !district.is_positioned()).map(|district| district.name.clone())
    }
}

impl Deref for Districts {
    type Target = [District];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Districts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
