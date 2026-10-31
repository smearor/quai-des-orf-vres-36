use crate::StationId;
use crate::StationKind;
use crate::StationName;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;
use tangible_objects_model::serde_pair;

/// One station row of the dataset.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationToml {
    /// Station number.
    pub id: StationId,
    /// Display name (proper noun, not localized).
    pub name: StationName,
    /// Position in map mm; TOML shape `pos_mm = [x, y]`.
    #[serde(rename = "pos_mm", with = "serde_pair")]
    pub pos: Vec2<Millimeters>,
    /// Special roles; empty means a plain station.
    #[serde(default)]
    pub kinds: BTreeSet<StationKind>,
}
