use crate::Neighbor;
use crate::StationId;
use crate::Transport;
use serde::Deserialize;
use serde::Serialize;

/// An undirected travel route between two stations.
///
/// Stored canonicalized with `from < to` - the game graph has no
/// one-way routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Edge {
    /// Station at one end (the lower id after canonicalization).
    pub from: StationId,
    /// Station at the other end.
    pub to: StationId,
    /// Transport mode this route is used with.
    pub transport: Transport,
}

impl Edge {
    /// Creates an edge, canonicalizing the endpoint order.
    pub fn new(a: StationId, b: StationId, transport: Transport) -> Self {
        let (from, to) = if a <= b { (a, b) } else { (b, a) };
        Self { from, to, transport }
    }

    /// The neighbor entry as seen from `from` (points to `to`).
    pub fn forward(&self) -> Neighbor {
        Neighbor {
            station: self.to,
            transport: self.transport,
        }
    }

    /// The neighbor entry as seen from `to` (points back to `from`).
    pub fn backward(&self) -> Neighbor {
        Neighbor {
            station: self.from,
            transport: self.transport,
        }
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
