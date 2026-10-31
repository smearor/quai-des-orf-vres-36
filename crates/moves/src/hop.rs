use qdo36_map::StationId;
use qdo36_map::Transport;
use serde::Deserialize;
use serde::Serialize;

/// One hop along a map edge: from one station to an adjacent station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hop {
    /// Station the move starts from.
    pub from: StationId,
    /// Station the move lands on.
    pub to: StationId,
    /// Transport mode of the travelled edge.
    pub transport: Transport,
}
