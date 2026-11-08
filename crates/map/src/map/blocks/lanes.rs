//! `HopLanes` - which lanes occupy one street hop.

use crate::LineName;
use std::collections::BTreeSet;

/// The lanes occupying one street hop: whether the taxi and the bus
/// corridors draw across it and which metro lines are routed over it.
/// Mirrors the render's parallel-transports lane semantics, so the
/// setback a block keeps from a street matches the corridor the
/// view draws.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HopLanes {
    /// At least one taxi edge draws over the hop.
    pub taxi: bool,
    /// At least one bus edge draws over the hop.
    pub bus: bool,
    /// Metro lines routed over the hop (empty when no metro runs
    /// across); unlined metro edges count separately.
    pub metro_lines: BTreeSet<LineName>,
    /// Metro edges without a line name routed over the hop - each
    /// draws its own lane under the metro overlay.
    pub unlined_metro: usize,
}

impl HopLanes {
    /// The lane count in the parallel-transports view: one lane per
    /// transport on the hop, the metro split per line when
    /// `metro_split` (the metro overlay) or a single generic metro
    /// lane otherwise.
    pub fn lanes(&self, metro_split: bool) -> usize {
        let metro = if metro_split {
            self.metro_lines.len() + self.unlined_metro
        } else {
            usize::from(self.metro_lines.len() + self.unlined_metro > 0)
        };
        usize::from(self.taxi) + usize::from(self.bus) + metro
    }
}
