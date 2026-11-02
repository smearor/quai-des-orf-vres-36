//! `SeatCount` - how many distinct start-station seats a game needs
//! or a map offers.

use serde::Deserialize;
use serde::Serialize;
use std::fmt;

use crate::DetectiveCount;

/// A count of deal seats: detectives plus Fugitive on the `needed`
/// side, available start stations on the `offered` side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SeatCount(u8);

impl SeatCount {
    /// A seat count.
    pub const fn new(count: u8) -> Self {
        Self(count)
    }

    /// A seat count measured off a collection; saturates at `u8::MAX`
    /// (only used for error reporting).
    pub fn of_len(len: usize) -> Self {
        Self(u8::try_from(len).unwrap_or(u8::MAX))
    }

    /// The count as `usize`, for collection APIs.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// The seats a game needs: one per detective plus Fugitive's
    /// hidden start.
    pub fn for_game(detectives: DetectiveCount) -> Self {
        Self(detectives.count() + 1)
    }

    /// The numeric count.
    pub const fn count(self) -> u8 {
        self.0
    }
}

impl fmt::Display for SeatCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
