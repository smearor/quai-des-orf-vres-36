use crate::map::District;
use crate::map::Edge;
use crate::map::MapMeta;
use crate::map::Park;
use crate::map::backdrop::Backdrop;
use crate::station::Station;
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
    pub stations: Vec<Station>,
    /// Edge list (undirected routes; endpoint order is irrelevant).
    pub edges: Vec<Edge>,
    /// Decorative backdrop layer (`[backdrop]` - optional,
    /// gameplay-inert).
    #[serde(default)]
    pub backdrop: Backdrop,
    /// Named city quarters (`[[districts]]` - optional): names the
    /// inner blocks carry, pinned geographically via `pos_mm`.
    #[serde(default)]
    pub districts: Vec<District>,
    /// Named parks (`[[parks]]` - optional): green spots large
    /// blocks keep free of houses.
    #[serde(default)]
    pub parks: Vec<Park>,
}
