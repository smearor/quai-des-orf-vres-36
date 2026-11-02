use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A ticket that pays for a move.
///
/// Taxi/bus/metro tickets are public for everyone; the black ticket
/// (any route incl. batobus) and the double ticket (two consecutive
/// moves) are Fugitive-only, as in Scotland Yard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TicketKind {
    /// Pays one taxi edge.
    Taxi,
    /// Pays one bus edge.
    Bus,
    /// Pays one metro edge.
    Metro,
    /// Fugitive-only: pays any route including batobus.
    Black,
    /// Fugitive-only: pays two consecutive hops in one turn.
    Double,
}

impl TicketKind {
    /// Whether the ticket is Fugitive-only.
    pub fn is_fugitive_only(self) -> bool {
        matches!(self, Self::Black | Self::Double)
    }
}

impl fmt::Display for TicketKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Taxi => "taxi",
            Self::Bus => "bus",
            Self::Metro => "metro",
            Self::Black => "black",
            Self::Double => "double",
        };
        f.write_str(name)
    }
}
