use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Unique identifier of a station on the Paris map.
///
/// Station numbers are displayed to the players (like the numbered
/// circles on the Scotland Yard board) and are what Monsieur Corbeau
/// types into the private keypad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StationId(u16);

impl StationId {
    /// Creates a station id from its numeric value.
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    /// Returns the numeric station number.
    pub const fn number(self) -> u16 {
        self.0
    }
}

impl From<u16> for StationId {
    fn from(id: u16) -> Self {
        Self::new(id)
    }
}

impl From<StationId> for u16 {
    fn from(id: StationId) -> Self {
        id.0
    }
}

impl fmt::Display for StationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtype_roundtrip() {
        let id = StationId::new(36);
        assert_eq!(id.number(), 36);
        assert_eq!(u16::from(id), 36);
        assert_eq!(StationId::from(36), id);
        assert_eq!(id.to_string(), "36");
    }
}
