pub mod count;
pub mod path;

use crate::LineName;
use crate::Neighbor;
use crate::StationId;
use crate::Transport;
pub use count::EdgeCount;
pub use path::EdgePath;
use serde::Deserialize;
use serde::Serialize;

/// An undirected travel route between two stations.
///
/// Stored canonicalized with `from < to` - the game graph has no
/// one-way routes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Edge {
    /// Station at one end (the lower id after canonicalization).
    pub from: StationId,
    /// Station at the other end.
    pub to: StationId,
    /// Transport mode this route is used with.
    pub transport: Transport,
    /// Transit line serving this route (metro edges only - the line's
    /// brand color styles the drawn route).
    #[serde(default)]
    pub line: Option<LineName>,
    /// Intermediate stations the drawn route passes through without
    /// stopping, in `from` -> `to` order. Empty routes along the
    /// street mesh automatically (`GameMap::route_on_surface`); set `via` to
    /// override a single edge's drawn path.
    #[serde(default)]
    pub via: Vec<StationId>,
    /// Draw the straight chord and skip street-mesh routing (metro
    /// edges only - relevant where `via` is empty and the mesh
    /// detour reads worse than the direct line).
    #[serde(default)]
    pub direct: bool,
}

impl Edge {
    /// Creates an edge, canonicalizing the endpoint order.
    pub fn new(a: StationId, b: StationId, transport: Transport) -> Self {
        let (from, to) = if a <= b { (a, b) } else { (b, a) };
        Self {
            from,
            to,
            transport,
            line: None,
            via: Vec::new(),
            direct: false,
        }
    }

    /// Whether the edge's drawn path routes over the surface
    /// street mesh: non-`direct` metro edges pass stations
    /// without stopping and follow the streets above; tunnels,
    /// `via` chains and direct chords do not.
    /// Whether the edge runs `transport`.
    pub fn is_transport(&self, transport: Transport) -> bool {
        self.transport == transport
    }

    pub fn uses_street_mesh(&self) -> bool {
        self.transport == Transport::Metro && self.via.is_empty() && !self.direct
    }

    /// The neighbor entry as seen from `from` (points to `to`).
    pub fn forward(&self) -> Neighbor {
        Neighbor {
            station: self.to,
            transport: self.transport,
            line: self.line.clone(),
        }
    }

    /// The neighbor entry as seen from `to` (points back to `from`).
    pub fn backward(&self) -> Neighbor {
        Neighbor {
            station: self.from,
            transport: self.transport,
            line: self.line.clone(),
        }
    }

    /// The edge's station chain `from` -> `via` -> `to` - the drawn
    /// route's stop order (`via` stations are passed through
    /// without stopping).
    pub fn station_chain(&self) -> impl Iterator<Item = StationId> + '_ {
        [self.from].into_iter().chain(self.via.iter().copied()).chain([self.to])
    }

    /// The full station chain `from -> via -> to` as an
    /// `EdgePath`.
    pub fn chain(&self) -> EdgePath {
        EdgePath(self.station_chain().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_endpoint_order() {
        let edge = Edge::new(StationId::new(9), StationId::new(3), Transport::Bus);
        assert_eq!(edge.from, StationId::new(3));
        assert_eq!(edge.to, StationId::new(9));
    }

    #[test]
    fn neighbor_entries_point_both_ways() {
        let edge = Edge::new(StationId::new(3), StationId::new(9), Transport::Bus);
        assert_eq!(edge.forward().station, StationId::new(9));
        assert_eq!(edge.backward().station, StationId::new(3));
        assert_eq!(edge.forward().transport, Transport::Bus);
    }
}
