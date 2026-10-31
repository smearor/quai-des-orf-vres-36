use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// A city's short flavor story (2-3 sentences of noir framing).
///
/// Optional dataset flavor shown during setup. Localization goes
/// through Fluent keys keyed by city name (`map-story-berlin`); the
/// dataset carries the English base text.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapStory(String);

impl MapStory {
    /// Whether the dataset carries a story.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The story as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MapStory {
    fn from(story: String) -> Self {
        Self(story)
    }
}

impl From<&str> for MapStory {
    fn from(story: &str) -> Self {
        Self(story.to_string())
    }
}

impl fmt::Display for MapStory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
