use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// The city's own name for a transport mode ("Métro", "Spreefähre").
///
/// Transit brands are proper nouns - not localized through Fluent.
/// Serializes transparently.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TransportName(String);

impl TransportName {
    /// The name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TransportName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for TransportName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl fmt::Display for TransportName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
