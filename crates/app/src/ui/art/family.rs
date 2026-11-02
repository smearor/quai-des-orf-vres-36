//! `ArtFamily` - the artwork namespace directory
//! (`resources/art/{family}/{city}/{style}/`).

/// The artwork family - matches the `qdo36-artwork` output
/// namespaces exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtFamily {
    /// Metro line badges (`ligne/`).
    Ligne,
    /// Detective pawn discs (`pawn/`).
    Pawn,
    /// Station marker pins (`station/`).
    Station,
    /// Ticket cards (`ticket/`).
    Ticket,
    /// Backdrop landmark pins (`landmarks/`).
    Landmarks,
    /// Corbeau art: emblem, reveal marker, shield zone (`corbeau/`).
    Corbeau,
    /// Map furniture: frame, compass (`map/`).
    Map,
    /// Splash title cards (`splash/`).
    Splash,
}

impl ArtFamily {
    /// The directory name under `resources/art/`.
    pub const fn dir_name(self) -> &'static str {
        match self {
            Self::Ligne => "ligne",
            Self::Pawn => "pawn",
            Self::Station => "station",
            Self::Ticket => "ticket",
            Self::Landmarks => "landmarks",
            Self::Corbeau => "corbeau",
            Self::Map => "map",
            Self::Splash => "splash",
        }
    }
}
