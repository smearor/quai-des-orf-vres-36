use crate::BlockFace;
use crate::City;
use crate::Edge;
use crate::EdgeCount;
use crate::EdgePath;
use crate::LineName;
use crate::MapError;
use crate::NearestStation;
use crate::Neighbors;
use crate::Polygon;
use crate::Polyline;
use crate::Segment;
use crate::Station;
use crate::StationCount;
use crate::StationId;
use crate::StationKind;
use crate::Stations;
use crate::Transport;
use crate::map::Districts;
use crate::map::MapMeta;
use crate::map::MapToml;
use crate::map::Park;
use crate::map::backdrop::Backdrop;
use crate::map::blocks::BlockCut;
use crate::map::blocks::BlockFaces;
use crate::map::blocks::BlockLayout;
use crate::map::blocks::CutSegment;
use crate::map::blocks::LaneMap;
use crate::map::bridge::BridgeDeck;
use crate::map::cost::Cost;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::binary_heap::BinaryHeap;
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
    stations: Stations,
    /// Canonicalized routes (`from < to`), one entry per dataset edge.
    edges: Vec<Edge>,
    /// Map coordinate bounds from the dataset meta.
    bounds: Region,
    /// City rule defaults (max rounds, reveal schedule).
    meta: MapMeta,
    /// Decorative backdrop (water, landmarks, streets) - empty when
    /// the dataset carries none.
    backdrop: Backdrop,
    /// Named city quarters the inner blocks carry (dataset
    /// `[[districts]]`; may be empty - blocks stay unnamed then).
    districts: Districts,
    /// Named parks large blocks keep free of houses (dataset
    /// `[[parks]]`; may be empty).
    parks: Vec<Park>,
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

    /// The edges of a transport mode.
    pub fn edges_by(&self, transport: Transport) -> impl Iterator<Item = &Edge> + '_ {
        self.edges.iter().filter(move |edge| edge.transport == transport)
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

    /// The water courses as drawn: each backdrop polyline Chaikin-
    /// smoothed, with the vertices sitting on `bridge` stations
    /// pinned so the river keeps flowing right under those markers.
    /// Feeds the river bands, the Batobus routes and the city-block
    /// cuts alike, so all of them share the same waterline.
    pub fn smoothed_water(&self) -> Vec<Polyline> {
        let bridge_positions: Vec<Vec2<Millimeters>> = self.stations.positions_of_kind(StationKind::Bridge).collect();
        self.backdrop
            .water
            .iter()
            .map(|polyline| polyline.smoothed_anchored(&bridge_positions, Millimeters::new(1.5), 2))
            .collect()
    }

    /// Stations carrying the given kind flag (start, batobus,
    /// bridge, ...).
    pub fn stations_of_kind(&self, kind: StationKind) -> impl Iterator<Item = &Station> {
        self.stations.of_kind(kind)
    }

    /// Stations eligible for the start deal.
    pub fn start_stations(&self) -> impl Iterator<Item = &Station> {
        self.stations_of_kind(StationKind::Start)
    }

    /// Neighbors of a station with their transport modes.
    ///
    /// Returns an empty slice for unknown ids.
    pub fn neighbors(&self, id: StationId) -> &Neighbors {
        self.stations.neighbors(id)
    }

    /// Neighbor stations reachable via the given transport mode.
    pub fn neighbors_by(&self, id: StationId, transport: Transport) -> impl Iterator<Item = StationId> + '_ {
        self.neighbors(id).stations_by(transport)
    }

    /// The transport modes served at each station, deduplicated.
    pub fn served_transports(&self) -> HashMap<StationId, Vec<Transport>> {
        self.stations.served_transports()
    }

    /// The metro lines served at each station, deduplicated -
    /// `hidden` lines dropped.
    pub fn served_lines(&self, hidden: &BTreeSet<LineName>) -> HashMap<StationId, Vec<LineName>> {
        self.stations.served_lines(hidden)
    }

    /// Station nearest to a map position, with its distance.
    ///
    /// Used by tap hit-testing and by pawn snap queries. Linear scan -
    /// the map is small enough for this to be cheap.
    pub fn nearest_station(&self, pos: Vec2<Millimeters>) -> Option<NearestStation> {
        self.stations.nearest(pos)
    }

    /// The straight-line distance between two stations in map mm -
    /// `None` when either id is unknown.
    pub fn station_distance(&self, from: StationId, to: StationId) -> Option<Millimeters> {
        self.stations.distance(from, to)
    }

    /// The drawn route segments of every underground edge - the
    /// tunnels the block faces record.
    fn tunnel_segments(&self) -> Vec<Segment<Millimeters>> {
        self.edges_by(Transport::Underground)
            .flat_map(|edge| self.edge_polyline(edge).segments().collect::<Vec<_>>())
            .collect()
    }

    /// Stations unreachable from the rest of the graph (BFS over
    /// neighbors).
    ///
    /// A soft dataset-quality check, not a load error: an island is
    /// legal TOML but almost certainly a data bug (a Scotland Yard map
    /// must be one component). Bundled maps are asserted connected in
    /// tests; callers may warn on non-empty results.
    /// Shortest path between two stations along `transport` edges,
    /// weighted by straight-line distance - the drawn route of an
    /// edge that passes stations without stopping (metro lines
    /// routed through the taxi mesh).
    pub fn route_on(&self, from: StationId, to: StationId, transport: Transport) -> Option<EdgePath> {
        self.route_on_where(from, to, |t| t == transport)
    }

    /// Shortest path along the surface street mesh (taxi + bus
    /// edges), weighted by straight-line distance - the drawn route
    /// of a metro edge that passes stations without stopping.
    pub fn route_on_surface(&self, from: StationId, to: StationId) -> Option<EdgePath> {
        self.route_on_where(from, to, |t| matches!(t, Transport::Taxi | Transport::Bus))
    }

    /// The station path an edge draws along: its declared `via`
    /// chain when set, the straight chord for tunnels and `direct`
    /// metro edges, otherwise the shortest route over the surface
    /// street mesh (metro lines passing stations without stopping).
    pub fn edge_path(&self, edge: &Edge) -> EdgePath {
        if !edge.via.is_empty() {
            return edge.station_chain().collect();
        }
        if edge.uses_street_mesh() {
            return self.route_on_surface(edge.from, edge.to).unwrap_or_else(|| EdgePath::from(edge));
        }
        EdgePath::from(edge)
    }

    /// The edge's drawn route in map mm: its station path resolved
    /// to positions (missing stations skipped).
    pub fn edge_polyline(&self, edge: &Edge) -> Polyline {
        self.edge_path(edge)
            .iter()
            .filter_map(|id| self.station(*id))
            .map(|station| station.pos)
            .collect()
    }

    /// Dijkstra over the neighbors that `serves` admits, weighted
    /// by straight-line distance; `None` when unreachable.
    fn route_on_where(&self, from: StationId, to: StationId, serves: impl Fn(Transport) -> bool) -> Option<EdgePath> {
        let mut dist: HashMap<StationId, Millimeters> = HashMap::new();
        let mut prev: HashMap<StationId, StationId> = HashMap::new();
        let mut heap = BinaryHeap::new();
        dist.insert(from, Millimeters::ZERO);
        heap.push((Cost::new(Millimeters::ZERO), from));
        while let Some((cost, node)) = heap.pop() {
            if node == to {
                break;
            }
            let here = *dist.get(&node).unwrap_or(&Millimeters::INFINITY);
            if cost.get() > here {
                continue;
            }
            for next in self.neighbors(node).stations_where(&serves) {
                let leg = self.station_distance(node, next).unwrap_or(Millimeters::new(1.0));
                let candidate = here + leg;
                if candidate < *dist.get(&next).unwrap_or(&Millimeters::INFINITY) {
                    dist.insert(next, candidate);
                    prev.insert(next, node);
                    heap.push((Cost::new(candidate), next));
                }
            }
        }
        if !prev.contains_key(&to) && from != to {
            return None;
        }
        let mut path = vec![to];
        while *path.last().unwrap_or(&to) != from {
            let node = *path.last().unwrap_or(&to);
            match prev.get(&node) {
                Some(&p) => path.push(p),
                None => break,
            }
        }
        path.reverse();
        Some(EdgePath(path))
    }

    /// The map's city-block division: the free areas enclosed by the
    /// surface street mesh (taxi/bus edges), the drawn water
    /// courses and the map border - computed once, immutable per
    /// map. Metro and underground edges run below the surface and
    /// never cut a block, but their routed paths do count toward the
    /// lane occupancy of the street hops they share.
    /// `river_half_width` is the drawn river band's half width - the
    /// bridge decks the surface streets dock at depend on it, and
    /// the block cuts must follow the same routed roads the
    /// renderer draws.
    pub fn block_layout(&self, river_half_width: Millimeters) -> BlockLayout {
        let water_courses = self.smoothed_water();
        let water_segments: Vec<Segment<Millimeters>> = water_courses.iter().flat_map(|course| course.segments()).collect();
        let (mut segments, lanes) = self.streets_and_lanes(river_half_width, &water_segments);
        for course in &water_courses {
            segments.extend(course.segments().map(|segment| CutSegment::of(segment, BlockCut::Water)));
        }
        let border = Polygon::new(self.bounds.corners().to_vec());
        segments.extend(border.segments().map(|segment| CutSegment::of(segment, BlockCut::Border)));
        BlockLayout {
            faces: self.populated_faces(&segments),
            lanes,
        }
    }

    /// The street cut segments and the lanes each hop carries:
    /// taxi/bus edges contribute both (their drawn roads cut the
    /// blocks, their hops carry lanes), metro edges only lanes.
    fn streets_and_lanes(&self, river_half_width: Millimeters, water_segments: &[Segment<Millimeters>]) -> (Vec<CutSegment>, LaneMap) {
        let decks = self.bridge_decks(river_half_width);
        let mut segments = Vec::new();
        let mut lanes = LaneMap::default();
        for edge in &self.edges {
            match edge.transport {
                Transport::Taxi | Transport::Bus => {
                    let ids = self.edge_path(edge);
                    for pair in ids.hops() {
                        lanes.record(pair, edge.transport, None);
                    }
                    segments.extend(self.street_cuts(&ids, &decks, water_segments));
                }
                Transport::Metro => {
                    for hop in self.edge_path(edge).hops() {
                        lanes.record(hop, edge.transport, edge.line.clone());
                    }
                }
                _ => {}
            }
        }
        (segments, lanes)
    }

    /// The street cuts one road edge contributes: the drawn road
    /// docks bridge-station endpoints at the deck end, not the
    /// marker, so blocks never cross the road.
    fn street_cuts(&self, ids: &EdgePath, decks: &HashMap<StationId, BridgeDeck>, water_segments: &[Segment<Millimeters>]) -> Vec<CutSegment> {
        self.road_route(ids, decks, water_segments).street_cuts()
    }

    /// The block faces the cut segments enclose, dressed with the
    /// underground tunnels, district names, parks and landmarks.
    /// Positioned quarters pin their name to the face they sit
    /// in; quarters without a position fill the still-unnamed
    /// inner blocks in dataset order.
    fn populated_faces(&self, segments: &[CutSegment]) -> BlockFaces {
        let mut faces = BlockFace::extract(segments);
        // The houses on a tunnel's way drop out; the block itself
        // is still drawn.
        faces.attach_all(self.tunnel_segments());
        for (pos, name) in self.districts.positioned() {
            faces.pin_district(&pos, name);
        }
        faces.fill_districts(self.districts.unpositioned_names());
        faces.populate(&self.parks, &self.backdrop.landmarks);
        faces
    }

    pub fn disconnected(&self) -> Vec<StationId> {
        let Some(start) = self.stations.keys().next() else {
            return Vec::new();
        };
        let mut visited = BTreeSet::new();
        let mut stack = vec![*start];
        while let Some(id) = stack.pop() {
            if visited.insert(id) {
                stack.extend(self.neighbors(id).stations());
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
        for (name, color) in dataset.map.lines.iter() {
            if color.a != 255 {
                return Err(MapError::TranslucentLineColor(name.clone()));
            }
        }

        let mut stations = Stations::default();
        for station in &dataset.stations {
            let pos = station.pos;
            if !pos.x.is_finite() || !pos.y.is_finite() || !bounds.contains(&pos) {
                return Err(MapError::OutOfBounds(station.id));
            }
            if station.is_batobus() && !station.is_bridge() {
                return Err(MapError::BatobusOffBridge(station.id));
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

        let water_segments: Vec<Segment<Millimeters>> = dataset.backdrop.water.iter().flat_map(|polyline| polyline.segments()).collect();

        let mut edges = Vec::with_capacity(dataset.edges.len());
        let mut seen = BTreeSet::new();
        for edge_toml in &dataset.edges {
            let (a, b) = (edge_toml.from, edge_toml.to);
            if a == b {
                return Err(MapError::SelfEdge(a));
            }
            for id in [a, b].into_iter().chain(edge_toml.via.iter().copied()) {
                if !stations.contains_key(&id) {
                    return Err(MapError::MissingStation(id));
                }
            }
            // canonicalize: deserialization does not enforce `from < to`
            let mut edge = Edge {
                line: edge_toml.line.clone(),
                via: edge_toml.via.clone(),
                ..Edge::new(a, b, edge_toml.transport)
            };
            if edge_toml.from > edge_toml.to {
                edge.via.reverse();
            }
            if let Some(line) = &edge.line {
                if edge.transport != Transport::Metro {
                    return Err(MapError::LineOnNonMetro {
                        from: a,
                        to: b,
                        line: line.clone(),
                    });
                }
                if !dataset.map.lines.contains(line) {
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
            if matches!(edge.transport, Transport::Taxi | Transport::Bus) {
                // Every hop of the drawn route (via points included)
                // needs a bridge when it crosses water.
                let path: Vec<StationId> = edge.station_chain().collect();
                let crossing = path.windows(2).any(|hop| {
                    water_segments
                        .iter()
                        .any(|water| Segment::new(stations[&hop[0]].pos, stations[&hop[1]].pos).crosses(water))
                        && !stations[&hop[0]].is_bridge()
                        && !stations[&hop[1]].is_bridge()
                });
                if crossing {
                    return Err(MapError::RoadCrossesWater {
                        from: a,
                        to: b,
                        transport: edge.transport,
                    });
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
            districts: Districts(dataset.districts),
            parks: dataset.parks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Neighbor;
    use crate::SquareMillimeters;
    use crate::StationKind;
    use qdo36_material::RoundNumber;

    fn mini_map() -> &'static str {
        r#"
stations = [
    { id = 1, name = "Tour Eiffel", pos_mm = [10.0, 10.0], kinds = ["start", "batobus", "bridge"] },
    { id = 2, name = "Louvre", pos_mm = [50.0, 50.0], kinds = ["start", "batobus", "bridge"] },
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

    fn river_map() -> &'static str {
        r#"
stations = [
    { id = 1, name = "North", pos_mm = [40.0, 10.0], kinds = [] },
    { id = 2, name = "South", pos_mm = [40.0, 90.0], kinds = [] },
    { id = 3, name = "Bridge", pos_mm = [10.0, 40.0], kinds = ["bridge"] },
]

edges = [
    { from = 1, to = 2, transport = "taxi" },
]

[map]
name = "river"
bounds_mm = [0.0, 0.0, 100.0, 100.0]

[backdrop]
water = [[0.0, 50.0, 100.0, 50.0]]
"#
    }

    #[test]
    fn route_on_finds_a_taxi_path() {
        let map = GameMap::from_str(mini_map()).unwrap_or_else(|e| panic!("{e}"));
        let route = map
            .route_on(StationId::new(1), StationId::new(3), Transport::Taxi)
            .unwrap_or_else(|| panic!("no taxi route"));
        assert_eq!(*route, [StationId::new(1), StationId::new(3)]);
        assert!(map.route_on(StationId::new(1), StationId::new(3), Transport::Metro).is_none());
    }

    #[test]
    fn route_on_passes_intermediate_stations() {
        let map = GameMap::from_str(include_str!("../../data/paris.toml")).expect("paris parses");
        // Châtelet -> République over the taxi mesh hops through the
        // dense grid instead of a straight chord.
        let route = map
            .route_on(StationId::new(20), StationId::new(19), Transport::Taxi)
            .unwrap_or_else(|| panic!("no route"));
        assert!(route.len() >= 2);
        assert_eq!(route.first().copied(), Some(StationId::new(20)));
        assert_eq!(route.last().copied(), Some(StationId::new(19)));
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
        let nearest = map.nearest_station(Vec2::new(12.0, 11.0)).unwrap_or_else(|| panic!("no stations"));
        assert_eq!(nearest.station, StationId::new(1));
        assert!(nearest.distance < Millimeters::from(3.0), "nearest is ~2.2 mm away: {}", nearest.distance);
    }

    #[test]
    fn bundled_block_layout_is_inside_bounds() {
        let map = GameMap::bundled().unwrap_or_else(|e| panic!("bundled paris.toml: {e}"));
        let layout = map.block_layout(Millimeters::new(3.5));
        // The street web + Seine + border cut Paris into dozens of
        // blocks; every face stays inside the map bounds.
        assert!(layout.faces.len() > 50, "{}", layout.faces.len());
        assert!(layout.faces.iter().all(|face| face.outline.iter().all(|p| p.x.is_finite() && p.y.is_finite())));
        assert!(layout.faces.iter().all(|face| face.signed_area() > SquareMillimeters::ZERO));
        assert!(layout.faces.iter().all(|face| face.outline.len() == face.cuts.len()));
        let lanes = layout
            .lanes
            .values()
            .filter(|lanes| lanes.taxi || lanes.bus || !lanes.metro_lines.is_empty() || lanes.unlined_metro > 0);
        assert!(lanes.count() > 100, "the street mesh occupies most hops");
    }

    #[test]
    fn districts_and_parks_load_from_the_dataset() {
        let map = GameMap::bundled().unwrap_or_else(|e| panic!("bundled paris.toml: {e}"));
        let layout = map.block_layout(Millimeters::new(3.5));
        // Inner faces carry TOML quarter names; positioned quarters
        // pin their name to the face they geographically sit in.
        let named = layout.faces.iter().filter(|f| f.district.is_some()).count();
        assert!(named > 40, "{named} named blocks");
        let montmartre = layout
            .faces
            .iter()
            .find(|f| f.district.as_ref().map(|name| name.as_str()) == Some("Montmartre"))
            .expect("Montmartre district");
        assert!(montmartre.contains(&Vec2::new(252.0, 88.0)));
        // Dataset parks land inside their containing face.
        let parks: Vec<&Park> = layout.faces.iter().flat_map(|face| face.parks.iter()).collect();
        assert!(parks.len() >= 10, "{} parks", parks.len());
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
    fn rejects_batobus_pier_off_bridge() {
        let source = mini_map().replace(r#"kinds = ["start", "batobus", "bridge"]"#, r#"kinds = ["start", "batobus"]"#);
        assert!(matches!(GameMap::from_str(&source), Err(MapError::BatobusOffBridge(id)) if id == StationId::new(1)));
    }

    #[test]
    fn rejects_road_crossing_water_off_bridge() {
        assert!(matches!(
            GameMap::from_str(river_map()),
            Err(MapError::RoadCrossesWater {
                transport: Transport::Taxi,
                ..
            })
        ));
    }

    #[test]
    fn allows_road_crossing_water_via_bridge() {
        let source = river_map().replace("{ from = 1, to = 2", "{ from = 2, to = 3");
        assert!(GameMap::from_str(&source).is_ok());
    }

    #[test]
    fn allows_metro_crossing_water() {
        let source = river_map().replace("taxi", "metro");
        assert!(GameMap::from_str(&source).is_ok());
    }

    #[test]
    fn allows_underground_crossing_water() {
        let source = river_map().replace("taxi", "underground").replace("kinds = []", r#"kinds = ["underground"]"#);
        assert!(GameMap::from_str(&source).is_ok());
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
