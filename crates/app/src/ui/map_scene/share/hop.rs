//! Who claims a slot on a route hop shared by several drawn edges.

use qdo36_map::Edge;
use qdo36_map::LineName;
use qdo36_map::Transport;

/// Who claims a slot on a hop shared by multiple drawn routes:
/// metro edges share per distinct line (same line, same slot),
/// underground tunnels share per edge (tunnels have no lines).
#[derive(Clone, PartialEq)]
pub enum HopShare {
    /// A metro edge claims the hop under its line name, so two
    /// edges of the same line share one slot instead of doubling.
    Metro(LineName),
    /// A line-less edge (underground tunnel) claims its own slot
    /// per edge index.
    Edge(usize),
}

impl HopShare {
    /// The share an edge claims on a hop: its metro line when it
    /// runs one, else its own edge identity.
    pub fn of(edge: &Edge, edge_i: usize) -> HopShare {
        match edge.transport {
            Transport::Metro => edge.line.clone().map(HopShare::Metro).unwrap_or(HopShare::Edge(edge_i)),
            _ => HopShare::Edge(edge_i),
        }
    }
}
