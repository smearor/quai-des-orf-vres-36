//! `Adjacency` - the undirected neighbor graph of one edge set.

use crate::Edge;
use crate::LinearityCause;
use crate::StationCount;
use crate::StationDegree;
use crate::StationId;
use std::collections::HashMap;
use std::collections::HashSet;

/// Undirected adjacency: station -> the stations its edges reach.
#[derive(Debug, Default)]
pub struct Adjacency {
    neighbors: HashMap<StationId, HashSet<StationId>>,
}

impl Adjacency {
    /// No stations in the graph.
    pub fn is_empty(&self) -> bool {
        self.neighbors.is_empty()
    }

    /// Edges incident to `station` (`0` when unserved).
    pub fn degree(&self, station: StationId) -> StationDegree {
        StationDegree::of_len(self.neighbors.get(&station).map_or(0, HashSet::len))
    }

    /// The stations the graph touches.
    pub fn served(&self) -> HashSet<StationId> {
        self.neighbors.keys().copied().collect()
    }

    /// Station + neighbor-set pairs (linearity checks iterate them).
    pub fn iter(&self) -> impl Iterator<Item = (&StationId, &HashSet<StationId>)> {
        self.neighbors.iter()
    }

    /// The linearity violations of the graph: none when it is a
    /// simple path (connected, degree <= 2, two endpoints).
    pub fn linearity_causes(&self) -> Vec<LinearityCause> {
        let mut causes = Vec::new();
        if self.is_empty() {
            return causes;
        }
        if self.component_sizes().len() > 1 {
            causes.push(LinearityCause::Disconnected);
        }
        let mut endpoints = 0u8;
        for (&station, neighbors) in self.iter() {
            match neighbors.len() {
                1 => endpoints += 1,
                d if d > 2 => causes.push(LinearityCause::Branched {
                    station,
                    degree: StationDegree::of_len(d),
                }),
                _ => {}
            }
        }
        if endpoints == 0 {
            causes.push(LinearityCause::Cyclic);
        } else if endpoints != 2 {
            causes.push(LinearityCause::EndpointCount {
                count: StationCount::new(u32::from(endpoints)),
            });
        }
        causes
    }

    /// Component sizes of the graph, sorted descending.
    pub fn component_sizes(&self) -> Vec<StationCount> {
        let mut seen = HashSet::new();
        let mut sizes = Vec::new();
        for &node in self.neighbors.keys() {
            if seen.contains(&node) {
                continue;
            }
            let mut size = 0u32;
            let mut stack = vec![node];
            while let Some(n) = stack.pop() {
                if seen.insert(n) {
                    size += 1;
                    for &next in self.neighbors.get(&n).into_iter().flatten() {
                        stack.push(next);
                    }
                }
            }
            sizes.push(StationCount::new(size));
        }
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        sizes
    }
}

impl<'a> FromIterator<&'a Edge> for Adjacency {
    /// Builds the undirected neighbor graph from edges.
    fn from_iter<T: IntoIterator<Item = &'a Edge>>(edges: T) -> Self {
        let mut neighbors: HashMap<StationId, HashSet<StationId>> = HashMap::new();
        for edge in edges {
            neighbors.entry(edge.from).or_default().insert(edge.to);
            neighbors.entry(edge.to).or_default().insert(edge.from);
        }
        Self { neighbors }
    }
}
