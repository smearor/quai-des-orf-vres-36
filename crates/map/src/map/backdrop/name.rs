use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A backdrop landmark's display name ("Tour Eiffel").
///
/// Proper nouns are not localized through Fluent. Serializes
/// transparently.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LandmarkName(String);

impl LandmarkName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for LandmarkName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for LandmarkName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl fmt::Display for LandmarkName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
