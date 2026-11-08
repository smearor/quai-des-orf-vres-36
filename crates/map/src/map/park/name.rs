use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A named green spot's name ("Jardin du Luxembourg",
/// "Buttes-Chaumont").
///
/// Parks name the green blobs a block keeps free of houses.
/// Proper nouns, not localized; serializes transparently.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParkName(String);

impl ParkName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ParkName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for ParkName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl fmt::Display for ParkName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
