use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Name of the city a map dataset describes ("paris", "berlin").
///
/// Serializes transparently as a plain string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CityName(String);

impl CityName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CityName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for CityName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl fmt::Display for CityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
