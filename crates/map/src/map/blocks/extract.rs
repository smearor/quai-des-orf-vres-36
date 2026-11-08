//! Planar face extraction: the map's cut lines (streets, river,
//! border) are split at every junction into a planar graph, then
//! each directed half-edge's left face is traced by always taking
//! the sharpest right turn at the next vertex.

use crate::Segment;
use crate::map::blocks::BlockCut;
use crate::map::blocks::BlockFace;
use crate::map::blocks::BlockFaces;
use crate::map::blocks::SquareMillimeters;
use crate::map::blocks::cut_segment::CutSegment;
use crate::map::blocks::splits::Splits;
use std::collections::HashMap;
use std::collections::HashSet;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Snap radius for sealing T-junctions and floating-point noise, in
/// mm: a segment endpoint within this distance of another segment's
/// interior docks onto it (piers sit ~1.5 mm off the water course).
const SNAP: Millimeters = Millimeters::new(2.5);

/// The smallest face area worth keeping (mm²) - dead-end spurs and
/// snapping slivers drop below it.
const MIN_AREA: SquareMillimeters = SquareMillimeters::new(4.0);

/// The union-find root of `index` (path halving).
fn root(parent: &mut [usize], mut index: usize) -> usize {
    while parent[index] != index {
        parent[index] = parent[parent[index]];
        index = parent[index];
    }
    index
}

/// Splits all segments at crossings and T-junctions, clusters the
/// endpoints into nodes and traces every half-edge's left face.
/// Returns the bounded faces (the unbounded outside and degenerate
/// slivers are dropped).
impl BlockFace {
    /// The faces the `segments` cut set encloses.
    pub fn extract(segments: &[CutSegment]) -> BlockFaces {
        let straights: Vec<Segment<Millimeters>> = segments.iter().map(Segment::from).collect();
        // Split parameters per segment: crossings split both sides,
        // T-junctions only the segment being docked onto.
        let mut splits = vec![Splits::default(); segments.len()];
        for (i, a) in straights.iter().enumerate() {
            for (j, b) in straights.iter().enumerate().skip(i + 1) {
                if let Some(point) = a.intersection(b) {
                    for (index, segment) in [(i, a), (j, b)] {
                        let direction = segment.to - segment.from;
                        let len_sq = direction.dot::<f64>(&direction).max(1e-18);
                        splits[index].push((point - segment.from).dot::<f64>(&direction) / len_sq);
                    }
                }
            }
            for (j, b) in straights.iter().enumerate() {
                if i == j {
                    continue;
                }
                for endpoint in [b.from, b.to] {
                    if let Some(projection) = a.interior_projection(&endpoint)
                        && projection.point.distance(&endpoint) < SNAP
                    {
                        splits[i].push(projection.fraction);
                    }
                }
            }
        }
        // Atomic fragments: each segment cut at its sorted split points.
        let mut fragments: Vec<(Vec2<Millimeters>, Vec2<Millimeters>, BlockCut)> = Vec::new();
        for (segment, ts) in segments.iter().zip(splits.iter_mut()) {
            ts.finish();
            let point = |t: f64| -> Vec2<Millimeters> {
                Vec2::new(segment.from.x + (segment.to.x - segment.from.x) * t, segment.from.y + (segment.to.y - segment.from.y) * t)
            };
            for (t0, t1) in ts.cuts() {
                let (from, to) = (point(t0), point(t1));
                if from.distance(&to).as_f64() > 1e-9 {
                    fragments.push((from, to, segment.cut));
                }
            }
        }
        // Cluster fragment endpoints into nodes (union-find over the
        // snap radius).
        let endpoints: Vec<Vec2<Millimeters>> = fragments.iter().flat_map(|(a, b, _)| [*a, *b]).collect();
        let mut parent: Vec<usize> = (0..endpoints.len()).collect();
        for i in 0..endpoints.len() {
            for j in i + 1..endpoints.len() {
                if endpoints[i].distance(&endpoints[j]) < SNAP {
                    let (ri, rj) = (root(&mut parent, i), root(&mut parent, j));
                    if ri != rj {
                        parent[ri] = rj;
                    }
                }
            }
        }
        // Margin preference when fragments coincide: `BlockCut`'s
        // order - street > water > border.
        let mut nodes: Vec<Vec2<Millimeters>> = Vec::new();
        let mut node_ids: HashMap<usize, usize> = HashMap::new();
        let mut adjacency: Vec<Vec<(usize, BlockCut, f64)>> = Vec::new();
        let mut seen_edges: HashMap<(usize, usize), BlockCut> = HashMap::new();
        for (index, (from, to, cut)) in fragments.iter().enumerate() {
            let (u, v) = (
                *node_ids.entry(root(&mut parent, index * 2)).or_insert_with(|| {
                    nodes.push(*from);
                    adjacency.push(Vec::new());
                    nodes.len() - 1
                }),
                *node_ids.entry(root(&mut parent, index * 2 + 1)).or_insert_with(|| {
                    nodes.push(*to);
                    adjacency.push(Vec::new());
                    nodes.len() - 1
                }),
            );
            if u == v {
                continue;
            }
            let key = if u < v { (u, v) } else { (v, u) };
            match seen_edges.get(&key) {
                Some(existing) if existing <= cut => continue,
                _ => {
                    seen_edges.insert(key, *cut);
                }
            }
            adjacency[u].push((v, *cut, (to.y - from.y).atan2(to.x - from.x)));
            adjacency[v].push((u, *cut, (from.y - to.y).atan2(from.x - to.x)));
        }
        for edges in adjacency.iter_mut() {
            edges.sort_by(|a, b| a.2.total_cmp(&b.2));
        }
        // Trace left faces: at each vertex take the edge just before the
        // twin in ascending-angle order (the sharpest right turn).
        let mut faces = Vec::new();
        let mut visited: HashSet<(usize, usize)> = HashSet::new();
        let max_steps = adjacency.iter().map(Vec::len).sum::<usize>() + 4;
        for u in 0..adjacency.len() {
            for edge_index in 0..adjacency[u].len() {
                let (mut from, mut to, mut cut) = {
                    let entry = adjacency[u][edge_index];
                    (u, entry.0, entry.1)
                };
                if visited.contains(&(from, to)) {
                    continue;
                }
                let start = (from, to);
                let mut vertices = vec![from];
                let mut cuts = Vec::new();
                loop {
                    visited.insert((from, to));
                    cuts.push(cut);
                    vertices.push(to);
                    let neighbors = &adjacency[to];
                    let twin = neighbors.iter().position(|(node, _, _)| *node == from).unwrap_or(0);
                    let (next, next_cut, _) = neighbors[(twin + neighbors.len() - 1) % neighbors.len()];
                    (from, to, cut) = (to, next, next_cut);
                    if (from, to) == start || cuts.len() > max_steps {
                        break;
                    }
                }
                vertices.pop();
                let mut face = BlockFace {
                    outline: vertices.iter().map(|node| nodes[*node]).collect(),
                    cuts,
                    tunnels: Vec::new(),
                    district: None,
                    parks: Vec::new(),
                    landmarks: Vec::new(),
                };
                face.remove_spikes();
                if face.outline.len() >= 3 && face.signed_area() > MIN_AREA {
                    faces.push(face);
                }
            }
        }
        BlockFaces(faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StationId;
    use crate::StationPair;

    fn street(x1: f64, y1: f64, x2: f64, y2: f64) -> CutSegment {
        CutSegment {
            from: Vec2::new(x1, y1),
            to: Vec2::new(x2, y2),
            cut: BlockCut::Street(StationPair::new(StationId::new(1), StationId::new(2))),
        }
    }

    fn border(x_max: f64, y_max: f64) -> Vec<CutSegment> {
        let corners = [Vec2::new(0.0, 0.0), Vec2::new(x_max, 0.0), Vec2::new(x_max, y_max), Vec2::new(0.0, y_max)];
        (0..4)
            .map(|i| CutSegment {
                from: corners[i],
                to: corners[(i + 1) % 4],
                cut: BlockCut::Border,
            })
            .collect()
    }

    #[test]
    fn a_cross_inside_the_border_gives_four_faces() {
        let mut segments = border(100.0, 100.0);
        segments.push(street(0.0, 50.0, 100.0, 50.0));
        segments.push(street(50.0, 0.0, 50.0, 100.0));
        let faces = BlockFace::extract(&segments);
        assert_eq!(faces.len(), 4, "{:?}", faces.iter().map(|f| f.outline.len()).collect::<Vec<_>>());
        assert!(faces.iter().all(|f| f.outline.len() == 4));
    }

    #[test]
    fn a_river_plus_one_street_split_the_map() {
        let mut segments = border(100.0, 100.0);
        segments.push(CutSegment {
            from: Vec2::new(-10.0, 50.0),
            to: Vec2::new(110.0, 50.0),
            cut: BlockCut::Water,
        });
        segments.push(street(50.0, 0.0, 50.0, 100.0));
        let faces = BlockFace::extract(&segments);
        assert_eq!(faces.len(), 4);
        let water_edges: usize = faces.iter().map(|f| f.cuts.iter().filter(|c| **c == BlockCut::Water).count()).sum();
        assert_eq!(water_edges, 4, "each of the two inner water fragments borders two faces");
    }

    #[test]
    fn a_dead_end_street_leaves_one_face() {
        let mut segments = border(100.0, 100.0);
        segments.push(street(0.0, 50.0, 40.0, 50.0));
        let faces = BlockFace::extract(&segments);
        assert_eq!(faces.len(), 1);
        // Border corners + the junction node where the spur docks.
        assert_eq!(faces[0].outline.len(), 5);
    }

    #[test]
    fn a_t_junction_seals_against_the_river() {
        let mut segments = border(100.0, 100.0);
        segments.push(CutSegment {
            from: Vec2::new(-10.0, 50.0),
            to: Vec2::new(110.0, 50.0),
            cut: BlockCut::Water,
        });
        // Pier access street ending 1.5mm short of the waterline:
        // it docks onto the river and splits the upper region.
        segments.push(street(50.0, 0.0, 50.0, 48.5));
        let faces = BlockFace::extract(&segments);
        assert_eq!(faces.len(), 3, "{:?}", faces.iter().map(|f| f.signed_area()).collect::<Vec<_>>());
    }
}
