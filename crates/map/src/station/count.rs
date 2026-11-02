//! `StationCount` - how many stations a map holds.

use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// The number of stations in a map dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StationCount(u32);

impl StationCount {
    /// A count of `n` stations.
    pub const fn new(n: u32) -> Self {
        Self(n)
    }

    /// The count from a collection length.
    pub fn of_len(len: usize) -> Self {
        Self(len.min(u32::MAX as usize) as u32)
    }

    /// The bare number, for arithmetic and comparisons.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for StationCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
