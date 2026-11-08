//! `TransportLayers` - per-transport edge visibility.
//!
//! Each transport is a separate layer so analysis views can isolate
//! e.g. the underground secret routes.

use qdo36_map::Transport;

/// Which transport edges render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportLayers {
    /// Taxi street edges.
    pub taxi: bool,
    /// Bus edges.
    pub bus: bool,
    /// Metro edges (any line).
    pub metro: bool,
    /// Batobus Seine edges.
    pub batobus: bool,
    /// Underground secret-route edges.
    pub underground: bool,
}

impl TransportLayers {
    /// Whether the edge's transport layer is on.
    pub fn visible(&self, transport: Transport) -> bool {
        match transport {
            Transport::Taxi => self.taxi,
            Transport::Bus => self.bus,
            Transport::Metro => self.metro,
            Transport::Batobus => self.batobus,
            Transport::Underground => self.underground,
        }
    }
}

impl Default for TransportLayers {
    fn default() -> Self {
        Self {
            taxi: true,
            bus: true,
            metro: true,
            batobus: true,
            underground: true,
        }
    }
}
