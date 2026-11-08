use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A transit line's designation ("U1", "M6", "S5").
///
/// Lines name the city's metro/rail routes; edges reference them to
/// carry line colors and names into rendering. Serializes
/// transparently.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LineName(String);

impl LineName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for LineName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for LineName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl fmt::Display for LineName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
