use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Stem of the tangible profile file anchoring the map
/// ("tour-de-eifel", "fernsehturm").
///
/// Resolved against the bundled profiles in `qdo36-tangibles`;
/// serializes transparently as a plain string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnchorProfile(String);

impl AnchorProfile {
    /// The profile name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AnchorProfile {
    fn from(profile: String) -> Self {
        Self(profile)
    }
}

impl From<&str> for AnchorProfile {
    fn from(profile: &str) -> Self {
        Self(profile.to_string())
    }
}

impl fmt::Display for AnchorProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
