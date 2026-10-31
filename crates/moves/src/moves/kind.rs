use crate::Hop;
use serde::Deserialize;
use serde::Serialize;

/// Shape of a move within one turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MoveKind {
    /// A single hop paid with one transport ticket.
    Single(Hop),
    /// Corbeau's double ticket: two consecutive hops in one turn.
    Double {
        /// First hop of the double move.
        first: Hop,
        /// Second hop, starting where `first` ended.
        second: Hop,
    },
}

impl MoveKind {
    /// The station the move ends on.
    pub fn destination(&self) -> qdo36_map::StationId {
        match self {
            Self::Single(hop) => hop.to,
            Self::Double { second, .. } => second.to,
        }
    }
}
