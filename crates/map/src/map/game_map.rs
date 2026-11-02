use crate::City;
use crate::Edge;
use crate::EdgeCount;
use crate::MapError;
use crate::Neighbor;
use crate::Station;
use crate::StationCount;
use crate::StationId;
use crate::Transport;
use crate::map::MapMeta;
use crate::map::MapToml;
use crate::map::backdrop::Backdrop;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::str::FromStr;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;
use tangible_objects_model::Vec2;

/// The validated Paris game graph.
///
/// Built once from the TOML dataset ([`MapToml`]); immutable for the
/// lifetime of a game. All coordinates are map millimeters.
#[derive(Debug, Clone, PartialEq)]
pub struct GameMap {
    /// Stations indexed by their station number (ordered by id).
    stations: BTreeMap<StationId, Station>,
    /// Canonicalized routes (`from < to`), one entry per dataset edge.
    edges: Vec<Edge>,
    /// Map coordinate bounds from the dataset meta.
    bounds: Region,
    /// City rule defaults (max rounds, reveal schedule).
    meta: MapMeta,
    /// Decorative backdrop (water, landmarks, streets) - empty when
    /// the dataset carries none.
    backdrop: Backdrop,
}

impl GameMap {
    /// The bundled dataset of a city.
    ///
    /// Compiled into the binary - the game works fully offline.
    ///
    /// # Errors
    ///
    /// [`MapError`] variants for an invalid bundled dataset (a broken
    /// build artifact, verified by tests).
    pub fn for_city(city: City) -> Result<Self, MapError> {
        Self::from_str(city.dataset())
    }

    /// The default city (Paris).
    ///
    /// # Errors
    ///
    /// Same as [`Self::for_city`].
    pub fn bundled() -> Result<Self, MapError> {
        Self::for_city(City::Paris)
    }

    /// Looks up a station by id.
    pub fn station(&self, id: StationId) -> Option<&Station> {
        self.stations.get(&id)
    }

    /// All stations, ordered by station number.
    pub fn stations(&self) -> impl Iterator<Item = &Station> {
        self.stations.values()
    }

    /// How many stations the map holds.
    pub fn station_count(&self) -> StationCount {
        StationCount::of_len(self.stations.len())
    }

    /// All canonicalized edges.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// How many edges the map holds.
    pub fn edge_count(&self) -> EdgeCount {
        EdgeCount::of_len(self.edges.len())
    }

    /// Map coordinate bounds.
    pub fn bounds(&self) -> Region {
        self.bounds
    }

    /// The dataset's city rule defaults (max rounds, reveal schedule).
    pub fn meta(&self) -> &MapMeta {
        &self.meta
    }

    /// The decorative backdrop layer (empty when the dataset carries
    /// none).
    pub fn backdrop(&self) -> &Backdrop {
        &self.backdrop
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

    /// Stations unreachable from the rest of the graph (BFS over
    /// neighbors).
    ///
    /// A soft dataset-quality check, not a load error: an island is
    /// legal TOML but almost certainly a data bug (a Scotland Yard map
    /// must be one component). Bundled maps are asserted connected in
    /// tests; callers may warn on non-empty results.
    pub fn disconnected(&self) -> Vec<StationId> {
        let Some(start) = self.stations.keys().next() else {
            return Vec::new();
        };
        let mut visited = BTreeSet::new();
        let mut stack = vec![*start];
        while let Some(id) = stack.pop() {
            if visited.insert(id) {
                stack.extend(self.neighbors(id).iter().map(|n| n.station));
            }
        }
        self.stations.keys().filter(|id| !visited.contains(id)).copied().collect()
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
        let bounds = dataset.map.bounds;
        if bounds.is_degenerate() {
            return Err(MapError::InvalidBounds);
        }
        if dataset.stations.is_empty() {
            return Err(MapError::EmptyMap);
        }

        let mut stations = BTreeMap::new();
        for station in &dataset.stations {
            let pos = station.pos;
            if !pos.x.is_finite() || !pos.y.is_finite() || !bounds.contains(&pos) {
                return Err(MapError::OutOfBounds(station.id));
            }
            if stations.insert(station.id, station.clone()).is_some() {
                return Err(MapError::DuplicateStationId(station.id));
            }
        }

        if let Some(anchor) = &dataset.map.anchor
            && !stations.contains_key(&anchor.station)
        {
            return Err(MapError::MissingStation(anchor.station));
        }
        if let Some(fugitive) = &dataset.map.fugitive
            && !stations.contains_key(&fugitive.station)
        {
            return Err(MapError::MissingStation(fugitive.station));
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
            // canonicalize: deserialization does not enforce `from < to`
            let edge = Edge {
                line: edge_toml.line.clone(),
                ..Edge::new(a, b, edge_toml.transport)
            };
            if let Some(line) = &edge.line {
                if edge.transport != Transport::Metro {
                    return Err(MapError::LineOnNonMetro {
                        from: a,
                        to: b,
                        line: line.clone(),
                    });
                }
                if !dataset.map.lines.contains_key(line) {
                    return Err(MapError::UnknownLine(line.clone()));
                }
            }
            if edge_toml.transport == Transport::Batobus {
                let both_piers = [edge.from, edge.to].iter().all(|id| stations.get(id).is_some_and(|s| s.is_batobus()));
                if !both_piers {
                    return Err(MapError::BatobusOnLand(edge.from, edge.to));
                }
            }
            if edge_toml.transport == Transport::Underground {
                let both_accesses = [edge.from, edge.to].iter().all(|id| stations.get(id).is_some_and(|s| s.is_underground()));
                if !both_accesses {
                    return Err(MapError::UndergroundOnSurface(edge.from, edge.to));
                }
            }
            if !seen.insert(edge.clone()) {
                return Err(MapError::DuplicateEdge {
                    from: edge.from,
                    to: edge.to,
                    transport: edge.transport,
                });
            }
            if let Some(station) = stations.get_mut(&edge.from) {
                station.neighbors.push(edge.forward());
            }
            if let Some(station) = stations.get_mut(&edge.to) {
                station.neighbors.push(edge.backward());
            }
            edges.push(edge);
        }

        Ok(Self {
            stations,
            edges,
            bounds,
            meta: dataset.map,
            backdrop: dataset.backdrop,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StationKind;
    use qdo36_material::RoundNumber;

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
name = "mini"
bounds_mm = [0.0, 0.0, 100.0, 100.0]
"#
    }

    #[test]
    fn parses_and_validates() {
        let map = GameMap::from_str(mini_map()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(map.station_count(), StationCount::new(3));
        assert_eq!(map.edge_count(), EdgeCount::new(3));
        assert_eq!(map.start_stations().count(), 3);
        let neighbors = map.neighbors(StationId::new(1));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&Neighbor {
            station: StationId::new(2),
            transport: Transport::Batobus,
            line: None
        }));
        assert_eq!(map.neighbors_by(StationId::new(3), Transport::Bus).collect::<Vec<_>>(), vec![StationId::new(2)]);
        let (nearest, distance) = map.nearest_station(Vec2::new(12.0, 11.0)).unwrap_or_else(|| panic!("no stations"));
        assert_eq!(nearest, StationId::new(1));
        assert!(distance < Millimeters::from(3.0), "nearest is ~2.2 mm away: {distance}");
    }

    #[test]
    fn bundled_dataset_is_valid() {
        let map = GameMap::bundled().unwrap_or_else(|e| panic!("bundled paris.toml: {e}"));
        assert!(map.station_count() >= StationCount::new(30), "curated map should have >= 30 stations");
        assert!(map.edges().iter().any(|e| e.transport == Transport::Batobus));
        assert!(map.edges().iter().all(|e| e.from < e.to), "edges are canonicalized");
    }

    #[test]
    fn all_bundled_cities_are_valid_and_named() {
        for city in City::ALL {
            let map = GameMap::for_city(city).unwrap_or_else(|e| panic!("{city}: {e}"));
            assert_eq!(map.meta().name.as_str(), city.name());
            assert!(map.station_count() > StationCount::new(0), "{city}");
            assert!(!map.meta().slogan.is_empty() && !map.meta().story.is_empty(), "{city} should carry flavor text");
            let fugitive = map.meta().fugitive.as_ref().unwrap_or_else(|| panic!("{city}: missing fugitive spec"));
            assert!(!fugitive.alias.is_empty(), "{city} should name its fugitive");
            assert!(map.disconnected().is_empty(), "{city} graph must be connected: {:?}", map.disconnected());
            assert!(!map.backdrop().is_empty(), "{city} should carry a backdrop layer");
            let anchor = map.meta().anchor.as_ref().unwrap_or_else(|| panic!("{city}: missing anchor spec"));
            assert!(!anchor.profile.as_str().is_empty() && !anchor.name.as_str().is_empty(), "{city}: incomplete anchor spec");
            // Viewport invariant: the drawable surroundings extend at
            // least 30 mm beyond every station, so panning keeps the
            // map on the table.
            let inner = map.bounds().inset(Millimeters::from(30.0));
            assert!(map.stations().all(|s| inner.contains(&s.pos)), "{city}: a station sits inside the 30 mm edge band");
        }
    }

    #[test]
    fn berlin_ships_custom_rule_defaults() {
        let map = GameMap::for_city(City::Berlin).unwrap_or_else(|e| panic!("{e}"));
        assert_ne!(map.meta().max_rounds, RoundNumber::DEFAULT_MAX);
        assert!(map.meta().reveal_rounds.iter().all(|r| *r <= map.meta().max_rounds));
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
    fn rejects_underground_edge_without_access() {
        let source = mini_map().replace("{ from = 1, to = 3, transport = \"taxi\" }", "{ from = 1, to = 3, transport = \"underground\" }");
        assert!(matches!(GameMap::from_str(&source), Err(MapError::UndergroundOnSurface(..))));
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
        let empty = "stations = []\nedges = []\n[map]\nname = \"empty\"\nbounds_mm = [0.0, 0.0, 100.0, 100.0]";
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
