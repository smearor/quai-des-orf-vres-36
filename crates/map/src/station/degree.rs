//! `StationDegree` - how many edges touch one station.

use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// The number of edges incident to a station (all transports).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StationDegree(u8);

impl StationDegree {
    /// A degree of `n` incident edges.
    pub const fn new(n: u8) -> Self {
        Self(n)
    }

    /// The degree from a collection length, saturating at `u8::MAX`.
    pub fn of_len(len: usize) -> Self {
        Self(len.min(u8::MAX as usize) as u8)
    }

    /// The bare number, for arithmetic and comparisons.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// No incident edges at all - an unreachable island station.
    pub const fn is_orphaned(self) -> bool {
        self.0 == 0
    }

    /// Exactly one incident edge - a dead end that traps whoever
    /// enters (Q9 leaf check).
    pub const fn is_leaf(self) -> bool {
        self.0 == 1
    }

    /// Fewer than two incident edges - a dealt detective would
    /// start trapped (Q10 underconnected start check).
    pub const fn is_underconnected(self) -> bool {
        self.0 < 2
    }
}

impl fmt::Display for StationDegree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
