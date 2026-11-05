//! `StationPair` - the canonical unordered key of a station hop.

use crate::station::StationId;

/// An unordered pair of station ids - the canonical key of a hop
/// shared by edges traversing it in either direction. `new`
/// orders the endpoints lower-first so `(a, b)` and `(b, a)`
/// share one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StationPair(StationId, StationId);

impl StationPair {
    /// Orders `a`/`b` lower-first.
    pub fn new(a: StationId, b: StationId) -> Self {
        if a < b { Self(a, b) } else { Self(b, a) }
    }

    /// The canonical lower endpoint.
    pub const fn a(self) -> StationId {
        self.0
    }

    /// The canonical higher endpoint.
    pub const fn b(self) -> StationId {
        self.1
    }

    /// `+1` when `from` is the canonical lower endpoint (the hop
    /// is traversed in key direction), `-1` otherwise.
    pub fn direction(self, from: StationId) -> f64 {
        if from == self.0 { 1.0 } else { -1.0 }
    }
}
