//! City vocabulary: bundled datasets and their names.

pub mod name;

pub use name::CityName;

use std::fmt;

/// A bundled city dataset.
///
/// Each variant embeds one TOML dataset; `GameMap::for_city` loads it.
/// Adding a city means: a `data/<city>.toml` file plus a variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum City {
    /// Paris - the default city.
    Paris,
    /// Berlin - longer game (28 rounds) with a custom reveal schedule.
    Berlin,
}

impl City {
    /// All bundled cities.
    pub const ALL: [City; 2] = [Self::Paris, Self::Berlin];

    /// The city's bundled TOML dataset.
    pub fn dataset(&self) -> &'static str {
        match self {
            Self::Paris => include_str!("../../data/paris.toml"),
            Self::Berlin => include_str!("../../data/berlin.toml"),
        }
    }

    /// The city's name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Paris => "paris",
            Self::Berlin => "berlin",
        }
    }
}

impl fmt::Display for City {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
