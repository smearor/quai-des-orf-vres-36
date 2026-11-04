//! Strategic importance of a station for movement-heavy roles.
//!
//! Score formula (documented, tuning-friendly):
//! `3 * reach + 2 * transports + degree`
//! - `reach`: distinct stations reachable in one move - how far the
//!   fugitive escapes per turn
//! - `transports`: served transports count - taxi-only stations are
//!   the least important
//! - `degree`: outgoing edges - raw branching factor

pub mod class;
pub mod score;
pub mod station_importance;

pub use class::ImportanceClass;
pub use score::ImportanceScore;
pub use station_importance::StationImportance;

use crate::GameMap;
use crate::StationId;
use std::collections::{BTreeSet, HashMap};

impl GameMap {
    /// Strategic importance per station: weighted score split into
    /// thirds (lower third `Minor`, middle `Typical`, top `Hub`).
    pub fn importance(&self) -> HashMap<StationId, StationImportance> {
        let mut scores: HashMap<StationId, ImportanceScore> = HashMap::new();
        for station in self.stations() {
            let mut degree = 0u32;
            let mut transports = BTreeSet::new();
            let mut reach = BTreeSet::new();
            for edge in self.edges() {
                if edge.from == station.id {
                    degree += 1;
                    transports.insert(edge.transport);
                    reach.insert(edge.to);
                } else if edge.to == station.id {
                    degree += 1;
                    transports.insert(edge.transport);
                    reach.insert(edge.from);
                }
            }
            let score = ImportanceScore(3 * reach.len() as u32 + 2 * transports.len() as u32 + degree);
            scores.insert(station.id, score);
        }
        // Tertile thresholds over the map's own distribution.
        let mut ordered: Vec<u32> = scores.values().map(|s| s.0).collect();
        ordered.sort_unstable();
        let n = ordered.len().max(1);
        let t1 = ordered[n / 3];
        let t2 = ordered[2 * n / 3];
        scores
            .into_iter()
            .map(|(id, score)| {
                let class = match score.0 {
                    s if s <= t1 => ImportanceClass::Minor,
                    s if s <= t2 => ImportanceClass::Typical,
                    _ => ImportanceClass::Hub,
                };
                (id, StationImportance { score, class })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn star_map() -> &'static str {
        r#"
stations = [
    { id = 1, name = "Hub", pos_mm = [50.0, 50.0], kinds = ["start"] },
    { id = 2, name = "Leaf A", pos_mm = [90.0, 50.0], kinds = [] },
    { id = 3, name = "Leaf B", pos_mm = [50.0, 90.0], kinds = [] },
    { id = 4, name = "Leaf C", pos_mm = [10.0, 50.0], kinds = [] },
]

edges = [
    { from = 1, to = 2, transport = "taxi" },
    { from = 1, to = 3, transport = "taxi" },
    { from = 1, to = 4, transport = "bus" },
    { from = 1, to = 2, transport = "metro", line = "M1" },
]

[map]
name = "star"
bounds_mm = [0.0, 0.0, 100.0, 100.0]

[map.lines]
M1 = '#FFCD00'
"#
    }

    #[test]
    fn importance_ranks_hub_above_leaves() {
        let map = GameMap::from_str(star_map()).expect("star parses");
        let importance = map.importance();
        let hub = importance.get(&StationId::new(1)).expect("hub scored");
        let leaf = importance.get(&StationId::new(3)).expect("leaf scored");
        assert!(hub.score > leaf.score);
        assert_eq!(hub.class, ImportanceClass::Hub);
    }
}
