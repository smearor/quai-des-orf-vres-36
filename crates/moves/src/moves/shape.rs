//! `MoveShape` - the hop-count shape of a move: one hop or a
//! two-hop double.

use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// How many hops a move covers.
///
/// Every [`crate::TicketKind`] expects exactly one shape: travel and
/// black tickets pay for `Single`, the double ticket requires
/// `Double`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MoveShape {
    /// A single hop along one edge.
    Single,
    /// Fugitive's double ticket: two consecutive hops.
    Double,
}

impl fmt::Display for MoveShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Single => "single-hop",
            Self::Double => "two-hop double",
        };
        f.write_str(name)
    }
}
