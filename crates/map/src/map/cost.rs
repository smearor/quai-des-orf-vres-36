//! `Cost` - an accumulated path cost for the min-heap search.

use std::cmp::Ordering;
use tangible_objects_model::Millimeters;

/// A routing cost (distance sum) with reversed `Ord` so the
/// `BinaryHeap` pops the cheapest path first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cost(Millimeters);

impl Cost {
    /// A cost of `c`.
    pub const fn new(c: Millimeters) -> Self {
        Self(c)
    }

    /// The bare cost value.
    pub const fn get(self) -> Millimeters {
        self.0
    }
}

impl Eq for Cost {}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cost {
    /// Reversed comparison: the binary heap behaves as a min-heap
    /// on path costs.
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}
