use crate::AnchorName;
use crate::AnchorProfile;
use crate::StationId;
use serde::Deserialize;
use serde::Serialize;

/// Which tangible anchors the map, and where it pins it down.
///
/// Each city names its own landmark: Paris uses the Tour Eiffel
/// (`tour-de-eifel`, pinned to the Tour Eiffel station), Berlin the
/// Fernsehturm on Alexanderplatz. The anchor station's `pos_mm` is
/// what [`crate::AnchorPose`]-driven transforms translate onto the
/// tangible's centroid.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AnchorSpec {
    /// Tangible profile stem inside the city's profile directory
    /// (`tour-de-eifel` in `paris/`, `fernsehturm` in `berlin/`).
    pub profile: AnchorProfile,
    /// Display name of the landmark ("Eiffelturm").
    pub name: AnchorName,
    /// The station the anchor pins down (its `pos_mm` becomes the
    /// transform's anchor point).
    pub station: StationId,
}
