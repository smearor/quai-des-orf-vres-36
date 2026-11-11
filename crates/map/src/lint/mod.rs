//! Dataset quality lint (`GameMap::lint`).
//!
//! Implements the WP10 quality criteria Q1-Q9 on a loaded map:
//! per-layer crossings, connectivity, line linearity, transfer
//! sharing, service coverage, leaf/duplicate detection and
//! per-transport density metrics.

pub mod adjacency;
pub mod component_count;
pub mod density;
pub mod edge_pair;
pub mod edge_set;
pub mod finding;
pub mod linearity_cause;
pub mod mean_degree;
pub mod missing_service;
pub mod report;

pub use adjacency::Adjacency;
pub use component_count::ComponentCount;
pub use density::TransportDensity;
pub use edge_pair::EdgePair;
pub use edge_set::EdgeSet;
pub use finding::Finding;
pub use linearity_cause::LinearityCause;
pub use mean_degree::MeanDegree;
pub use missing_service::MissingService;
pub use report::LintReport;

use crate::GameMap;
use crate::LineName;
use crate::Segment;
use crate::StationCount;
use crate::StationId;
use crate::Transport;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use std::collections::BTreeMap;
use std::collections::HashSet;

impl GameMap {
    /// Station clearance (Q14): an edge passing within this distance
    /// of a foreign station reads as running through it.
    const STATION_CLEARANCE: Millimeters = Millimeters::new(4.0);

    /// Lints the dataset against the WP10 quality criteria Q1-Q9.
    pub fn lint(&self) -> LintReport {
        let mut findings = Vec::new();
        self.check_crossings(&mut findings);
        self.check_components(&mut findings);
        self.check_lines(&mut findings);
        self.check_coverage(&mut findings);
        self.check_topology_details(&mut findings);
        self.check_station_clearance(&mut findings);
        LintReport {
            findings,
            density: self.density(),
        }
    }

    /// Edges of one transport layer.
    fn edges_of(&self, transport: Transport) -> EdgeSet<'_> {
        self.edges_by(transport).collect()
    }

    /// Q1: same-layer crossings are defects (metro exempt - lines
    /// may cross each other; a same-line self-crossing still reports).
    fn check_crossings(&self, findings: &mut Vec<Finding>) {
        let groups: BTreeMap<Transport, EdgeSet> = self.edges().iter().fold(BTreeMap::new(), |mut g, e| {
            g.entry(e.transport).or_default().push(e);
            g
        });
        for (transport, edges) in groups {
            for (i, a) in edges.iter().enumerate() {
                for b in &edges[i + 1..] {
                    // A shared station on either drawn path is a
                    // junction, not a crossing.
                    let path_a: Vec<StationId> = a.station_chain().collect();
                    let path_b: Vec<StationId> = b.station_chain().collect();
                    if path_a.iter().any(|id| path_b.contains(id)) {
                        continue;
                    }
                    // Different metro lines may legitimately cross
                    // (tunnels); only same-line self-crossings report.
                    if transport == Transport::Metro && a.line != b.line {
                        continue;
                    }
                    let crosses = path_a.windows(2).any(|hop_a| {
                        let seg_a = Segment::new(self.pos(hop_a[0]), self.pos(hop_a[1]));
                        path_b
                            .windows(2)
                            .any(|hop_b| seg_a.crosses(&Segment::new(self.pos(hop_b[0]), self.pos(hop_b[1]))))
                    });
                    if crosses {
                        findings.push(Finding::Crossing {
                            transport,
                            first: EdgePair {
                                from: a.from,
                                to: a.to,
                                line: a.line.clone(),
                            },
                            second: EdgePair {
                                from: b.from,
                                to: b.to,
                                line: b.line.clone(),
                            },
                        });
                    }
                }
            }
        }
    }

    /// Q14: a foreign edge passing within the station disc reads as
    /// "the route runs through the station without serving it".
    /// `via` waypoints are declared pass-throughs, not served stops.
    fn check_station_clearance(&self, findings: &mut Vec<Finding>) {
        for edge in self.edges() {
            let path: Vec<StationId> = edge.station_chain().collect();
            let on_path: HashSet<StationId> = path.iter().copied().collect();
            for station in self.stations() {
                if on_path.contains(&station.id) {
                    continue;
                }
                let distance_mm = path
                    .windows(2)
                    .filter_map(|hop| Segment::new(self.pos(hop[0]), self.pos(hop[1])).interior_distance(&station.pos))
                    .fold(Millimeters::INFINITY, Millimeters::min);
                if distance_mm < Self::STATION_CLEARANCE {
                    findings.push(Finding::StationOnEdge {
                        transport: edge.transport,
                        edge: EdgePair {
                            from: edge.from,
                            to: edge.to,
                            line: edge.line.clone(),
                        },
                        station: station.id,
                        distance_mm,
                    });
                }
            }
        }
    }

    /// A station's map-mm position as a raw pair.
    fn pos(&self, id: StationId) -> Vec2<Millimeters> {
        self.station(id).map(|s| s.pos).unwrap_or(Vec2::ZERO)
    }

    /// Q2: each transport net must be one component. Underground
    /// tunnels are exempt: separate Égouts systems are by design
    /// (local shortcuts, not one connected net).
    fn check_components(&self, findings: &mut Vec<Finding>) {
        for transport in Transport::ALL {
            if transport == Transport::Underground {
                continue;
            }
            let adj = self.edges_of(transport).adjacency();
            if adj.is_empty() {
                continue;
            }
            let sizes = adj.component_sizes();
            if sizes.len() > 1 {
                findings.push(Finding::Disconnected {
                    transport,
                    components: ComponentCount::of_len(sizes.len()),
                    sizes,
                });
            }
        }
    }

    /// Q3/Q4/Q7: metro lines linear + mutually reachable; batobus
    /// chain linear.
    fn check_lines(&self, findings: &mut Vec<Finding>) {
        // Per-line edge sets.
        let mut lines: BTreeMap<LineName, EdgeSet> = BTreeMap::new();
        let mut line_stations: BTreeMap<LineName, HashSet<StationId>> = BTreeMap::new();
        for edge in self.edges() {
            if let Some(line) = &edge.line {
                lines.entry(line.clone()).or_default().push(edge);
                let entry = line_stations.entry(line.clone()).or_default();
                entry.insert(edge.from);
                entry.insert(edge.to);
            }
        }
        for (line, edges) in &lines {
            let adj = edges.adjacency();
            for cause in adj.linearity_causes() {
                findings.push(Finding::LineNotLinear { line: line.clone(), cause });
            }
            // Q4: the line must share at least one station with
            // another line (transfer hub).
            let stations = &line_stations[line];
            let shared = line_stations
                .iter()
                .any(|(other, their)| other != line && stations.intersection(their).next().is_some());
            if !shared {
                findings.push(Finding::IsolatedLine(line.clone()));
            }
        }
        // Q7: the batobus chain is one linear route.
        let batobus_adj = self.edges_of(Transport::Batobus).adjacency();
        for cause in batobus_adj.linearity_causes() {
            findings.push(Finding::BatobusNotLinear { cause });
        }
    }

    /// Q5/Q6: metro stations need taxi+bus; ≥90% of bus stations
    /// need taxi.
    fn check_coverage(&self, findings: &mut Vec<Finding>) {
        let taxi = self.edges_of(Transport::Taxi).served();
        let bus = self.edges_of(Transport::Bus).served();
        let metro = self.edges_of(Transport::Metro).served();
        for station in &metro {
            let missing = match (taxi.contains(station), bus.contains(station)) {
                (true, true) => continue,
                (false, true) => MissingService::Taxi,
                (true, false) => MissingService::Bus,
                (false, false) => MissingService::Both,
            };
            findings.push(Finding::MetroStationUnserved { station: *station, missing });
        }
        if !bus.is_empty() {
            let covered = StationCount::of_len(bus.intersection(&taxi).count());
            let total = StationCount::of_len(bus.len());
            if (covered.get() as f64) < (total.get() as f64) * 0.9 {
                findings.push(Finding::BusCoverageBelow {
                    taxi_covered: covered,
                    bus_total: total,
                });
            }
        }
    }

    /// Q9/Q10/Q11: leaf stations, duplicate positions, start-station
    /// degree and the taxi >= bus >= metro density order.
    fn check_topology_details(&self, findings: &mut Vec<Finding>) {
        let adj = Adjacency::from_iter(self.edges().iter());
        let starts: HashSet<StationId> = self.start_stations().map(|s| s.id).collect();
        for station in self.stations() {
            let degree = adj.degree(station.id);
            if degree.is_orphaned() {
                findings.push(Finding::OrphanedStation(station.id));
            }
            if degree.is_leaf() {
                findings.push(Finding::LeafStation(station.id));
            }
            if starts.contains(&station.id) && degree.is_underconnected() {
                findings.push(Finding::UnderconnectedStart { station: station.id, degree });
            }
        }
        for (sparser, denser) in [(Transport::Metro, Transport::Bus), (Transport::Bus, Transport::Taxi)] {
            if self.edges_of(sparser).len() > self.edges_of(denser).len() {
                findings.push(Finding::DensityInversion { sparser, denser });
            }
        }
        let mut seen: BTreeMap<(i64, i64), StationId> = BTreeMap::new();
        for station in self.stations() {
            let key = ((station.pos.x * 1000.0) as i64, (station.pos.y * 1000.0) as i64);
            if let Some(other) = seen.insert(key, station.id) {
                findings.push(Finding::DuplicatePosition(other, station.id));
            }
        }
    }

    /// Q8: per-transport density metrics (informational).
    fn density(&self) -> BTreeMap<Transport, TransportDensity> {
        Transport::ALL
            .iter()
            .filter_map(|t| {
                let edges = self.edges_of(*t);
                if edges.is_empty() {
                    return None;
                }
                let count = edges.len();
                let stations = edges.served_count();
                Some((
                    *t,
                    TransportDensity {
                        edges: count,
                        stations_served: stations,
                        avg_degree: MeanDegree::of(count, stations),
                    },
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// A dataset satisfying every criterion.
    const CLEAN: &str = r##"
stations = [
    { id = 1, name = "A", pos_mm = [10.0, 10.0], kinds = ["start", "underground"] },
    { id = 2, name = "B", pos_mm = [50.0, 10.0], kinds = ["start"] },
    { id = 3, name = "Hub", pos_mm = [90.0, 10.0], kinds = ["start"] },
    { id = 4, name = "C", pos_mm = [130.0, 60.0], kinds = ["start", "underground"] },
    { id = 5, name = "D", pos_mm = [50.0, 150.0], kinds = ["batobus", "bridge"] },
    { id = 6, name = "E", pos_mm = [90.0, 150.0], kinds = ["batobus", "underground", "bridge"] },
    { id = 7, name = "F", pos_mm = [130.0, 150.0], kinds = ["batobus", "bridge"] },
]

edges = [
    { from = 1, to = 2, transport = "taxi" },
    { from = 2, to = 3, transport = "taxi" },
    { from = 3, to = 4, transport = "taxi" },
    { from = 1, to = 5, transport = "taxi" },
    { from = 2, to = 6, transport = "taxi" },
    { from = 3, to = 7, transport = "taxi" },
    { from = 5, to = 6, transport = "taxi" },
    { from = 6, to = 7, transport = "taxi" },
    { from = 2, to = 3, transport = "bus" },
    { from = 3, to = 4, transport = "bus" },
    { from = 3, to = 7, transport = "bus" },
    { from = 2, to = 3, transport = "metro", line = "M1" },
    { from = 3, to = 4, transport = "metro", line = "M1" },
    { from = 3, to = 7, transport = "metro", line = "M2" },
    { from = 5, to = 6, transport = "batobus" },
    { from = 6, to = 7, transport = "batobus" },
    { from = 1, to = 4, transport = "underground" },
]

[map]
name = "Fixture"
bounds_mm = [0.0, 0.0, 400.0, 400.0]
max_rounds = 24
reveal_rounds = [3, 8, 13, 18, 24]

[map.lines]
M1 = "#FF0000"
M2 = "#0000FF"
"##;

    /// Parses a fixture and returns its lint findings.
    /// Appends an edge line after `anchor` (keeping the commas).
    fn with_edge(source: &str, anchor: &str, extra: &str) -> String {
        source.replace(anchor, &format!("{anchor}\n    {extra},"))
    }

    /// Parses a fixture and returns its lint findings.
    fn findings_of(source: &str) -> Vec<Finding> {
        GameMap::from_str(source).expect("fixture parses").lint().findings
    }

    #[test]
    fn a_clean_map_lints_without_findings() {
        let report = GameMap::from_str(CLEAN).expect("fixture parses").lint();
        assert!(report.is_clean(), "unexpected findings: {:?}", report.findings);
        assert_eq!(report.density.len(), 5);
    }

    #[test]
    fn orphaned_station_reports() {
        let source = CLEAN.replace(
            "{ id = 7, name = \"F\", pos_mm = [130.0, 150.0], kinds = [\"batobus\", \"bridge\"] },",
            "{ id = 7, name = \"F\", pos_mm = [130.0, 150.0], kinds = [\"batobus\", \"bridge\"] },\n    { id = 8, name = \"Orphan\", pos_mm = [200.0, 200.0], kinds = [] },",
        );
        let findings = findings_of(&source);
        assert!(findings.iter().any(|f| matches!(f, Finding::OrphanedStation(id) if *id == StationId::new(8))), "{findings:?}");
    }

    #[test]
    fn station_on_foreign_edge_reports() {
        // Station 4 back onto the 1-2-3 line: the underground edge
        // 1->4 now runs straight through stations 2 and 3.
        let source = CLEAN.replace("pos_mm = [130.0, 60.0]", "pos_mm = [130.0, 10.0]");
        let findings = findings_of(&source);
        assert!(
            findings
                .iter()
                .any(|f| matches!(f, Finding::StationOnEdge { transport: Transport::Underground, station, .. } if *station == StationId::new(2))),
            "{findings:?}"
        );
    }

    #[test]
    fn crossing_edges_report() {
        let source = with_edge(
            CLEAN,
            "{ from = 1, to = 4, transport = \"underground\" },",
            "{ from = 1, to = 6, transport = \"underground\" }",
        );
        let source = with_edge(
            &source,
            "{ from = 1, to = 6, transport = \"underground\" },",
            "{ from = 2, to = 5, transport = \"underground\" }",
        );
        let source = source.replace("kinds = [\"start\"] },\n    { id = 3", "kinds = [\"start\", \"underground\"] },\n    { id = 3");
        let source = source.replace(
            "kinds = [\"batobus\", \"bridge\"] },\n    { id = 6",
            "kinds = [\"batobus\", \"underground\", \"bridge\"] },\n    { id = 6",
        );
        let findings = findings_of(&source);
        assert!(
            findings.iter().any(|f| matches!(
                f,
                Finding::Crossing {
                    transport: Transport::Underground,
                    ..
                }
            )),
            "{findings:?}"
        );
    }

    #[test]
    fn a_disconnected_net_reports() {
        // An extra bus pair on the pier stations splits the bus net
        // into two components.
        let source = with_edge(CLEAN, "{ from = 3, to = 7, transport = \"bus\" },", "{ from = 5, to = 6, transport = \"bus\" }");
        let findings = findings_of(&source);
        assert!(findings.iter().any(|f| matches!(f, Finding::Disconnected { transport: Transport::Bus, .. })), "{findings:?}");
    }

    #[test]
    fn a_branched_line_reports() {
        // M2 = {3-7, 3-6, 3-2}: three edges from hub 3 - a fork.
        let source = with_edge(
            CLEAN,
            "{ from = 3, to = 7, transport = \"metro\", line = \"M2\" },",
            "{ from = 3, to = 6, transport = \"metro\", line = \"M2\" }",
        );
        let source = with_edge(
            &source,
            "{ from = 3, to = 6, transport = \"metro\", line = \"M2\" },",
            "{ from = 2, to = 3, transport = \"metro\", line = \"M2\" }",
        );
        // Metro stations need bus service: 6 gets a bus edge.
        let source = with_edge(&source, "{ from = 2, to = 3, transport = \"bus\" },", "{ from = 2, to = 6, transport = \"bus\" }");
        let findings = findings_of(&source);
        assert!(
            findings.iter().any(|f| matches!(
                f,
                Finding::LineNotLinear {
                    cause: LinearityCause::Branched { .. },
                    ..
                }
            )),
            "{findings:?}"
        );
    }

    #[test]
    fn an_isolated_line_reports() {
        // M2 moves to 5-6 (the batobus stations): taxi+bus cover them
        // but no other line shares a station.
        let source = CLEAN.replace(
            "{ from = 3, to = 7, transport = \"metro\", line = \"M2\" },",
            "{ from = 5, to = 6, transport = \"metro\", line = \"M2\" },",
        );
        let source = with_edge(&source, "{ from = 3, to = 7, transport = \"bus\" },", "{ from = 5, to = 6, transport = \"bus\" }");
        let findings = findings_of(&source);
        assert!(findings.iter().any(|f| matches!(f, Finding::IsolatedLine(_))), "{findings:?}");
    }

    #[test]
    fn underconnected_starts_report() {
        // Strip station 1 down to a single edge: a start trap.
        let source = CLEAN.replace(
            "    { from = 1, to = 2, transport = \"taxi\" },\n    { from = 2, to = 3, transport = \"taxi\" },",
            "    { from = 2, to = 3, transport = \"taxi\" },",
        );
        let source = source.replace("    { from = 1, to = 5, transport = \"taxi\" },\n", "");
        let findings = findings_of(&source);
        assert!(findings.iter().any(|f| matches!(f, Finding::UnderconnectedStart { .. })), "{findings:?}");
    }

    #[test]
    fn density_inversion_reports() {
        // Overload the bus layer past the taxi layer.
        let mut source = CLEAN.to_string();
        for pair in [(1, 5), (1, 6), (2, 4), (3, 5), (3, 6), (4, 7), (5, 7)] {
            source = with_edge(
                &source,
                "{ from = 6, to = 7, transport = \"taxi\" },",
                &format!("{{ from = {}, to = {}, transport = \"bus\" }}", pair.0, pair.1),
            );
        }
        let findings = findings_of(&source);
        assert!(
            findings.iter().any(|f| matches!(
                f,
                Finding::DensityInversion {
                    sparser: Transport::Bus,
                    denser: Transport::Taxi
                }
            )),
            "{findings:?}"
        );
    }

    #[test]
    fn paris_is_clean() {
        let map = GameMap::from_str(include_str!("../../data/paris.toml")).expect("paris parses");
        let report = map.lint();
        assert!(report.is_clean(), "{:?}", report.findings);
    }
}
