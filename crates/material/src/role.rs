use crate::PawnId;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A seat at the table.
///
/// `Fugitive` is the fleeing master criminal (Paris-themed stand-in for
/// Scotland Yard's Mister X); detectives hunt him.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Monsieur Fugitive - the hidden player.
    Fugitive,
    /// One of the detectives chasing Fugitive.
    Detective(PawnId),
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fugitive => f.write_str("fugitive"),
            Self::Detective(pawn) => write!(f, "detective-{pawn}"),
        }
    }
}
