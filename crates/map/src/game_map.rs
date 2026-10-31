use crate::Bounds;
use crate::Edge;
use crate::MapError;
use crate::Neighbor;
use crate::Station;
use crate::StationId;
use crate::Transport;
use crate::dataset::MapToml;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::str::FromStr;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// The validated Paris game graph.
///
/// Built once from the TOML dataset ([`MapToml`]); immutable for the
/// lifetime of a game. All coordinates are map millimeters.
#[derive(Debug, Clone, PartialEq)]
pub struct GameMap {
    stations: BTreeMap<StationId, Station>,
    edges: Vec<Edge>,
    bounds: Bounds,
}

impl GameMap {
    /// The bundled `paris.toml` dataset.
    ///
    /// Compiled into the binary - the game works fully offline.
    pub fn bundled() -> Result<Self, MapError> {
        Self::from_str(include_str!("../data/paris.toml"))
    }

    /// Looks up a station by id.
    pub fn station(&self, id: StationId) -> Option<&Station> {
        self.stations.get(&id)
    }

    /// All stations, ordered by station number.
    pub fn stations(&self) -> impl Iterator<Item = &Station> {
        self.stations.values()
    }

    /// All canonicalized edges.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Map coordinate bounds.
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    /// Stations eligible for the start deal.
    pub fn start_stations(&self) -> impl Iterator<Item = &Station> {
        self.stations.values().filter(|s| s.is_start())
    }

    /// Neighbors of a station with their transport modes.
    ///
    /// Returns an empty slice for unknown ids.
    pub fn neighbors(&self, id: StationId) -> &[Neighbor] {
        self.stations.get(&id).map_or(&[], |s| s.neighbors.as_slice())
    }

    /// Neighbor stations reachable via the given transport mode.
    pub fn neighbors_by(&self, id: StationId, transport: Transport) -> impl Iterator<Item = StationId> + '_ {
        self.neighbors(id)
            .iter()
            .filter_map(move |neighbor| (neighbor.transport == transport).then_some(neighbor.station))
    }

    /// Station nearest to a map position, with its distance.
    ///
    /// Used by tap hit-testing and by pawn snap queries. Linear scan -
    /// the map is small enough for this to be cheap.
    pub fn nearest_station(&self, pos: Vec2<Millimeters>) -> Option<(StationId, Millimeters)> {
        self.stations
            .values()
            .map(|station| (station.id, pos.distance(&station.pos)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}

impl FromStr for GameMap {
    type Err = MapError;

    /// Parses and validates a TOML map dataset.
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let dataset: MapToml = toml::from_str(source)?;
        Self::try_from(dataset)
    }
}

impl TryFrom<MapToml> for GameMap {
    type Error = MapError;

    fn try_from(dataset: MapToml) -> Result<Self, Self::Error> {
        let bounds = Bounds {
            min: Vec2::new(dataset.map.bounds_mm[0], dataset.map.bounds_mm[1]),
            max: Vec2::new(dataset.map.bounds_mm[2], dataset.map.bounds_mm[3]),
        };
        bounds.validate()?;
        if dataset.stations.is_empty() {
            return Err(MapError::EmptyMap);
        }

        let mut stations = BTreeMap::new();
        for station_toml in &dataset.stations {
            let id = station_toml.id;
            let pos = Vec2::new(station_toml.pos_mm[0], station_toml.pos_mm[1]);
            if !pos.x.is_finite() || !pos.y.is_finite() || !bounds.contains(pos) {
                return Err(MapError::OutOfBounds(id));
            }
            let station = Station {
                id,
                name: station_toml.name.clone(),
                pos,
                kinds: station_toml.kinds.clone(),
                neighbors: Vec::new(),
            };
            if stations.insert(id, station).is_some() {
                return Err(MapError::DuplicateStationId(id));
            }
        }

        let mut edges = Vec::with_capacity(dataset.edges.len());
        let mut seen = BTreeSet::new();
        for edge_toml in &dataset.edges {
            let (a, b) = (edge_toml.from, edge_toml.to);
            if a == b {
                return Err(MapError::SelfEdge(a));
            }
            for id in [a, b] {
                if !stations.contains_key(&id) {
                    return Err(MapError::MissingStation(id));
                }
            }
            let edge = Edge::new(a, b, edge_toml.transport);
            if edge_toml.transport == Transport::Batobus {
                let both_piers = [edge.from, edge.to].iter().all(|id| stations.get(id).is_some_and(|s| s.is_batobus()));
                if !both_piers {
                    return Err(MapError::BatobusOnLand(edge.from, edge.to));
                }
            }
            if !seen.insert(edge) {
                return Err(MapError::DuplicateEdge {
                    from: edge.from,
                    to: edge.to,
                    transport: edge.transport,
                });
            }
            edges.push(edge);
            if let Some(station) = stations.get_mut(&edge.from) {
                station.neighbors.push(edge.forward());
            }
            if let Some(station) = stations.get_mut(&edge.to) {
                station.neighbors.push(edge.backward());
            }
        }

        Ok(Self { stations, edges, bounds })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StationKind;

    fn mini_map() -> &'static str {
        r#"
stations = [
    { id = 1, name = "Tour Eiffel", pos_mm = [10.0, 10.0], kinds = ["start", "batobus"] },
    { id = 2, name = "Louvre", pos_mm = [50.0, 50.0], kinds = ["start", "batobus"] },
    { id = 3, name = "Opéra", pos_mm = [80.0, 20.0], kinds = ["start"] },
]

edges = [
    { from = 1, to = 2, transport = "batobus" },
    { from = 1, to = 3, transport = "taxi" },
    { from = 2, to = 3, transport = "bus" },
]

[map]
bounds_mm = [0.0, 0.0, 100.0, 100.0]
"#
    }

    #[test]
    fn parses_and_validates() {
        let map = GameMap::from_str(mini_map()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(map.stations().count(), 3);
        assert_eq!(map.edges().len(), 3);
        assert_eq!(map.start_stations().count(), 3);
        let neighbors = map.neighbors(StationId::new(1));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&Neighbor {
            station: StationId::new(2),
            transport: Transport::Batobus
        }));
        assert_eq!(map.neighbors_by(StationId::new(3), Transport::Bus).collect::<Vec<_>>(), vec![StationId::new(2)]);
        let (nearest, distance) = map.nearest_station(Vec2::new(12.0, 11.0)).unwrap_or_else(|| panic!("no stations"));
        assert_eq!(nearest, StationId::new(1));
        assert!(distance < Millimeters::from(3.0), "nearest is ~2.2 mm away: {distance}");
    }

    #[test]
    fn bundled_dataset_is_valid() {
        let map = GameMap::bundled().unwrap_or_else(|e| panic!("bundled paris.toml: {e}"));
        assert!(map.stations().count() >= 30, "curated map should have >= 30 stations");
        assert!(map.edges().iter().any(|e| e.transport == Transport::Batobus));
        assert!(map.edges().iter().all(|e| e.from < e.to), "edges are canonicalized");
    }

    #[test]
    fn rejects_duplicate_station_ids() {
        let source = mini_map().replace("id = 3", "id = 1");
        assert!(matches!(GameMap::from_str(&source), Err(MapError::DuplicateStationId(id)) if id == StationId::new(1)));
    }

    #[test]
    fn rejects_missing_station() {
        let source = mini_map().replace("to = 3, transport = \"bus\"", "to = 99, transport = \"bus\"");
        assert!(matches!(GameMap::from_str(&source), Err(MapError::MissingStation(id)) if id == StationId::new(99)));
    }

    #[test]
    fn rejects_self_edge() {
        let source = mini_map().replace("from = 1, to = 2", "from = 1, to = 1");
        assert!(matches!(GameMap::from_str(&source), Err(MapError::SelfEdge(id)) if id == StationId::new(1)));
    }

    #[test]
    fn rejects_duplicate_edge() {
        let source = mini_map().replace("{ from = 2, to = 3, transport = \"bus\" }", "{ from = 1, to = 3, transport = \"taxi\" }");
        assert!(matches!(GameMap::from_str(&source), Err(MapError::DuplicateEdge { .. })));
    }

    #[test]
    fn rejects_batobus_edge_on_land() {
        let source = mini_map().replace("{ from = 1, to = 3, transport = \"taxi\" }", "{ from = 1, to = 3, transport = \"batobus\" }");
        assert!(matches!(GameMap::from_str(&source), Err(MapError::BatobusOnLand(..))));
    }

    #[test]
    fn rejects_out_of_bounds_station() {
        let source = mini_map().replace("pos_mm = [80.0, 20.0]", "pos_mm = [200.0, 20.0]");
        assert!(matches!(GameMap::from_str(&source), Err(MapError::OutOfBounds(id)) if id == StationId::new(3)));
    }

    #[test]
    fn rejects_invalid_and_empty() {
        let invalid_bounds = mini_map().replace("[0.0, 0.0, 100.0, 100.0]", "[0.0, 0.0, 0.0, 0.0]");
        assert!(matches!(GameMap::from_str(&invalid_bounds), Err(MapError::InvalidBounds)));
        let empty = "stations = []\nedges = []\n[map]\nbounds_mm = [0.0, 0.0, 100.0, 100.0]";
        assert!(matches!(GameMap::from_str(empty), Err(MapError::EmptyMap)));
    }

    #[test]
    fn rejects_malformed_toml() {
        assert!(matches!(GameMap::from_str("not = [toml"), Err(MapError::Parse(_))));
    }

    #[test]
    fn kinds_flags() {
        let map = GameMap::from_str(mini_map()).unwrap_or_else(|e| panic!("{e}"));
        let tour = map.station(StationId::new(1)).unwrap_or_else(|| panic!("station 1"));
        assert!(tour.is_start() && tour.is_batobus());
        assert!(tour.kinds.contains(&StationKind::Batobus));
        assert!(map.station(StationId::new(99)).is_none());
    }
}
