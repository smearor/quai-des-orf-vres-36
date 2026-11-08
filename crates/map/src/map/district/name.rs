use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A city quarter's name ("Montmartre", "Quartier Latin").
///
/// Districts name the inner blocks the houses are drawn into so
/// players can communicate positions. Proper nouns, not
/// localized; serializes transparently.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DistrictName(String);

impl DistrictName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DistrictName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for DistrictName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl fmt::Display for DistrictName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
