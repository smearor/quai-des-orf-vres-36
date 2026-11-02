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

impl std::str::FromStr for TicketKind {
    type Err = UnknownTicketKind;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "taxi" => Ok(Self::Taxi),
            "bus" => Ok(Self::Bus),
            "metro" => Ok(Self::Metro),
            "black" => Ok(Self::Black),
            "double" => Ok(Self::Double),
            other => Err(UnknownTicketKind(other.to_string())),
        }
    }
}

/// Error for an unrecognized ticket kind name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown ticket kind {0:?}")]
pub struct UnknownTicketKind(pub String);
