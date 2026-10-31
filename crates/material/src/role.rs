use crate::PawnId;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A seat at the table.
///
/// `Corbeau` is the fleeing master criminal (Paris-themed stand-in for
/// Scotland Yard's Mister X); detectives hunt him.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Monsieur Corbeau - the hidden player.
    Corbeau,
    /// One of the detectives chasing Corbeau.
    Detective(PawnId),
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corbeau => f.write_str("corbeau"),
            Self::Detective(pawn) => write!(f, "detective-{pawn}"),
        }
    }
}
