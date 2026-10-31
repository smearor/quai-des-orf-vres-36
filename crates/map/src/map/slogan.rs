use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A city's one-line flavor slogan ("Paris holds its breath.").
///
/// Optional dataset flavor shown on the splash/setup screens.
/// Localization goes through Fluent keys keyed by city name
/// (`map-slogan-paris`); the dataset carries the English base text.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapSlogan(String);

impl MapSlogan {
    /// Whether the dataset carries a slogan.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The slogan as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MapSlogan {
    fn from(slogan: String) -> Self {
        Self(slogan)
    }
}

impl From<&str> for MapSlogan {
    fn from(slogan: &str) -> Self {
        Self(slogan.to_string())
    }
}

impl fmt::Display for MapSlogan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
