//! `EdgeSet` - a borrowed edge collection of one lint view.

use crate::Edge;
use crate::EdgeCount;
use crate::StationCount;
use crate::StationId;
use crate::lint::Adjacency;
use std::collections::HashSet;
use std::ops::Deref;

/// A set of edge references - per-transport, per-line or per
/// finding group. `Deref` exposes it as a `&[&Edge]` slice for
/// indexed iteration; the domain methods build graph views.
#[derive(Default)]
pub struct EdgeSet<'a> {
    edges: Vec<&'a Edge>,
}

impl<'a> EdgeSet<'a> {
    /// No edges in the set.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Edges in the set.
    pub fn len(&self) -> EdgeCount {
        EdgeCount::of_len(self.edges.len())
    }

    /// Appends one edge reference (map-fold collection).
    pub fn push(&mut self, edge: &'a Edge) {
        self.edges.push(edge);
    }

    /// The undirected neighbor graph of the set.
    pub fn adjacency(&self) -> Adjacency {
        Adjacency::from_iter(self.edges.iter().copied())
    }

    /// The stations the set touches.
    pub fn served(&self) -> HashSet<StationId> {
        self.adjacency().served()
    }

    /// Stations any edge of the set touches, as a count.
    pub fn served_count(&self) -> StationCount {
        StationCount::of_len(self.served().len())
    }
}

impl<'a> FromIterator<&'a Edge> for EdgeSet<'a> {
    /// Collects borrowed edges into the set.
    fn from_iter<T: IntoIterator<Item = &'a Edge>>(edges: T) -> Self {
        Self {
            edges: edges.into_iter().collect(),
        }
    }
}

impl<'a> Deref for EdgeSet<'a> {
    type Target = [&'a Edge];

    /// Borrowed edges as a slice (indexing + iteration).
    fn deref(&self) -> &Self::Target {
        &self.edges
    }
}
