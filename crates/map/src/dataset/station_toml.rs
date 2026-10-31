use crate::StationId;
use crate::StationKind;
use crate::StationName;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;

/// One station row of the dataset.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationToml {
    /// Station number.
    pub id: StationId,
    /// Display name (proper noun, not localized).
    pub name: StationName,
    /// Position in map mm as `[x, y]`.
    pub pos_mm: [f64; 2],
    /// Special roles; empty means a plain station.
    #[serde(default)]
    pub kinds: BTreeSet<StationKind>,
}
