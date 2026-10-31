use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// The city's fugitive alias ("Monsieur Corbeau", "Dagobert").
///
/// Thematic flavor like the anchor landmark: each city names its own
/// fugitive. The internal identity stays `Role::Fugitive`; this is
/// the display name. Serializes transparently.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FugitiveAlias(String);

impl FugitiveAlias {
    /// Whether the dataset carries an alias.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The alias as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for FugitiveAlias {
    fn from(alias: String) -> Self {
        Self(alias)
    }
}

impl From<&str> for FugitiveAlias {
    fn from(alias: &str) -> Self {
        Self(alias.to_string())
    }
}

impl fmt::Display for FugitiveAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
