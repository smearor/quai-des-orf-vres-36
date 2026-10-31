use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Display name of a station.
///
/// Station names are proper nouns ("Tour Eiffel", "Sacré-Cœur") and are
/// deliberately not localized - they are the same in every language.
/// Serializes transparently as a plain string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StationName(String);

impl StationName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for StationName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for StationName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl AsRef<str> for StationName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_serde() {
        let name = StationName::from("Tour Eiffel");
        assert_eq!(serde_json::to_string(&name).unwrap_or_default(), "\"Tour Eiffel\"");
        assert_eq!(name.as_str(), "Tour Eiffel");
        assert_eq!(name.to_string(), "Tour Eiffel");
    }
}
