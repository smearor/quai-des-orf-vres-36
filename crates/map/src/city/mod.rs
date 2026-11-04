//! City vocabulary: bundled datasets and their names.

pub mod name;

pub use name::CityName;

use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

/// A bundled city dataset.
///
/// Each variant embeds one TOML dataset; `GameMap::for_city` loads it.
/// Adding a city means: a `data/<city>.toml` file plus a variant here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum City {
    /// Paris - the default city.
    #[default]
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

    /// The city's dataset on disk for hot reloads - resolves next
    /// to the crate in a dev checkout; `None` in installed layouts
    /// where only the bundled copy exists.
    pub fn dataset_path(&self) -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data").join(format!("{}.toml", self.name()));
        path.is_file().then_some(path)
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

/// Parse error for [`FromStr`](std::str::FromStr) on [`City`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown city {0:?} - expected one of: paris, berlin")]
pub struct CityParseError(String);

impl std::str::FromStr for City {
    type Err = CityParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .find(|city| city.name() == source)
            .copied()
            .ok_or_else(|| CityParseError(source.to_string()))
    }
}
