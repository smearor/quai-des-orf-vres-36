use crate::PawnId;
use serde::Deserialize;
use serde::Serialize;

/// A seat at the table.
///
/// `Corbeau` is the fleeing master criminal (Paris-themed stand-in for
/// Scotland Yard's Mister X); detectives hunt him.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Monsieur Corbeau - the hidden player.
    Corbeau,
    /// One of the detectives chasing Corbeau.
    Detective(PawnId),
}
