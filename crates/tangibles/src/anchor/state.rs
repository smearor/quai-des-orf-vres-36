use crate::AnchorPose;
use serde::Deserialize;
use serde::Serialize;

/// Where the map anchor currently is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorState {
    /// No tower on the table (or `TangibleMode::NoTangibles`): the map
    /// sits at a fixed default pose so rendering and snapping still work.
    Virtual(AnchorPose),
    /// The Eiffel tower tangible is tracked; the map follows it live.
    Tracked(AnchorPose),
}

impl AnchorState {
    /// A centered, upright default pose used in touch-only mode or
    /// before the tower is placed.
    pub fn default_virtual() -> Self {
        Self::Virtual(AnchorPose::ZERO)
    }

    /// The current pose regardless of tracking state.
    pub fn pose(&self) -> AnchorPose {
        match self {
            Self::Virtual(pose) | Self::Tracked(pose) => *pose,
        }
    }
}
