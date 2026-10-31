use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Identifier of a detective pawn (its seat color index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PawnId(u8);

impl PawnId {
    /// Creates a pawn id.
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    /// Returns the numeric pawn index.
    pub const fn index(self) -> u8 {
        self.0
    }
}

impl fmt::Display for PawnId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
