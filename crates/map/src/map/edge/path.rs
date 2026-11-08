//! `EdgePath` - the station chain an edge's route follows.

use crate::map::Edge;
use crate::station::StationId;
use crate::station::StationPair;
use std::ops::Deref;

/// The ordered stations an edge's drawn route passes: its `via`
/// chain when declared, otherwise the surface-route waypoints
/// (or the straight chord for tunnels and direct metro edges).
/// Derefs to the `[StationId]` slice.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EdgePath(pub Vec<StationId>);

impl From<&Edge> for EdgePath {
    /// The straight chord `from -> to` - the path when no `via`
    /// chain or street-mesh routing applies.
    fn from(edge: &Edge) -> Self {
        Self(vec![edge.from, edge.to])
    }
}

impl EdgePath {
    /// The path's stations as a slice.
    pub fn as_slice(&self) -> &[StationId] {
        &self.0
    }

    /// The consecutive station pairs along the path - the hops
    /// the edge is actually drawn as.
    pub fn hops(&self) -> impl Iterator<Item = StationPair> + '_ {
        self.0.windows(2).map(|pair| StationPair::new(pair[0], pair[1]))
    }

    /// The paths chained into one station sequence by shared
    /// endpoints, reversing pieces that meet the chain backwards.
    /// Pieces connected to neither end stay dropped.
    pub fn joined(paths: Vec<EdgePath>) -> EdgePath {
        let mut chain = paths.first().cloned().unwrap_or_default();
        let mut rest: Vec<EdgePath> = paths.into_iter().skip(1).collect();
        while let Some(index) = rest.iter().position(|piece| piece.first() == chain.last() || piece.last() == chain.last()) {
            let mut piece = rest.swap_remove(index);
            if piece.first() != chain.last() {
                piece.0.reverse();
            }
            chain.0.extend(piece.0.into_iter().skip(1));
        }
        chain
    }
}

impl Deref for EdgePath {
    type Target = [StationId];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<StationId> for EdgePath {
    fn from_iter<I: IntoIterator<Item = StationId>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a EdgePath {
    type Item = &'a StationId;
    type IntoIter = std::slice::Iter<'a, StationId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
