use crate::map::dataset::EdgeToml;
use crate::map::dataset::MapMeta;
use crate::map::dataset::StationToml;
use serde::Deserialize;
use serde::Serialize;

/// Root of the TOML map dataset (`data/paris.toml`).
///
/// Pure deserialization shape - [`crate::GameMap`] is produced by
/// validating this structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapToml {
    /// Map-wide metadata.
    pub map: MapMeta,
    /// Station table.
    pub stations: Vec<StationToml>,
    /// Edge list (undirected routes; endpoint order is irrelevant).
    pub edges: Vec<EdgeToml>,
}
