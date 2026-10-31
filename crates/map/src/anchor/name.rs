use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Display name of the anchor landmark ("Eiffelturm", "Fernsehturm").
///
/// A proper noun - not localized. Serializes transparently.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnchorName(String);

impl AnchorName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AnchorName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for AnchorName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl fmt::Display for AnchorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
