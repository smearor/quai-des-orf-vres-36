use crate::map::backdrop::LandmarkKind;
use crate::map::backdrop::LandmarkName;
use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;
use tangible_objects_model::serde_pair;

/// A decorative landmark marker on the map (Tour Eiffel,
/// Fernsehturm, Brandenburger Tor, ...).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Landmark {
    /// Display name (proper noun, not localized).
    pub name: LandmarkName,
    /// Position in map millimeters; TOML shape `pos_mm = [x, y]`.
    #[serde(rename = "pos_mm", with = "serde_pair")]
    pub pos: Vec2<Millimeters>,
    /// What kind of landmark this is (drives the icon).
    pub kind: LandmarkKind,
}
