use serde::Deserialize;
use serde::Serialize;

/// Map-wide metadata of the dataset.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapMeta {
    /// Map coordinate bounds as `[min_x, min_y, max_x, max_y]` in mm.
    pub bounds_mm: [f64; 4],
}
