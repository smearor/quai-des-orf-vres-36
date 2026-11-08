//! `Streets` - the decorative streets of one block.

use crate::map::blocks::Angle;
use crate::map::blocks::decoration::Street;
use std::ops::Deref;
use std::ops::DerefMut;

/// Directions considered too parallel for a second street (~36°).
const STREET_PARALLEL: Angle = Angle::new(0.65);

/// The decorative streets of a block as a set - Derefs to the
/// `Vec<Street>`.
#[derive(Debug, Clone, Default)]
pub struct Streets(pub Vec<Street>);

impl Streets {
    /// Whether any street runs (nearly) parallel to `direction` -
    /// a second street must run in another direction so the two
    /// never cross.
    pub fn has_parallel(&self, direction: Angle) -> bool {
        self.0
            .iter()
            .filter_map(|street| street.segment())
            .map(|segment| segment.direction())
            .any(|first_direction| (direction.as_f64() - first_direction.as_f64()).sin().abs() < STREET_PARALLEL.as_f64().sin())
    }

    /// Whether any street crosses `candidate`.
    pub fn has_crossing(&self, candidate: &Street) -> bool {
        self.0.iter().any(|street| street.crosses(candidate))
    }
}

impl Deref for Streets {
    type Target = Vec<Street>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Streets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a Streets {
    type Item = &'a Street;
    type IntoIter = std::slice::Iter<'a, Street>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Streets {
    type Item = Street;
    type IntoIter = std::vec::IntoIter<Street>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
