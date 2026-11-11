//! `SceneEntity`/`EntityKind` - one drawn thing's position in both
//! coordinate spaces (`scene_layout` reply).

use qdo36_map::StationId;
use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Vec2;

/// What kind of drawable an entity is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityKind {
    /// A play-relevant station.
    Station,
    /// A decorative backdrop landmark.
    Landmark,
}

/// One drawn entity with its position in map-mm **and** widget-px -
/// the two coordinates visual debugging correlates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    /// Station or landmark.
    pub kind: EntityKind,
    /// Display name (station/landmark name).
    pub name: String,
    /// The station id (`None` for landmarks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station: Option<StationId>,
    /// Position in map millimeters.
    pub mm: Vec2<Millimeters>,
    /// Position in widget pixels at the current view.
    pub px: Vec2<Pixels>,
}
