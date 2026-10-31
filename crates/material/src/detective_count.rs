use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use thiserror::Error;

/// The number of detectives hunting Corbeau.
///
/// Scotland Yard plays with 2-5 detectives (the sixth seat is Corbeau
/// himself).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct DetectiveCount(u8);

impl DetectiveCount {
    /// Fewest detectives a game supports.
    pub const MIN: u8 = 2;
    /// Most detectives a game supports.
    pub const MAX: u8 = 5;

    /// The numeric count.
    pub const fn count(self) -> u8 {
        self.0
    }
}

/// Error for an out-of-range detective count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("detective count {0} is out of range {min}..={max}", min = DetectiveCount::MIN, max = DetectiveCount::MAX)]
pub struct DetectiveCountError(pub u8);

impl TryFrom<u8> for DetectiveCount {
    type Error = DetectiveCountError;

    fn try_from(count: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&count) {
            Ok(Self(count))
        } else {
            Err(DetectiveCountError(count))
        }
    }
}

impl From<DetectiveCount> for u8 {
    fn from(count: DetectiveCount) -> Self {
        count.0
    }
}

impl fmt::Display for DetectiveCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_range_and_rejects_outside() {
        for count in DetectiveCount::MIN..=DetectiveCount::MAX {
            assert!(DetectiveCount::try_from(count).is_ok());
        }
        for count in [0, 1, 6, 255] {
            assert!(matches!(DetectiveCount::try_from(count), Err(DetectiveCountError(c)) if c == count));
        }
    }
}
