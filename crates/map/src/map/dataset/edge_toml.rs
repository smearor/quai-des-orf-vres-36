use crate::StationId;
use crate::Transport;
use serde::Deserialize;
use serde::Serialize;

/// One edge row of the dataset.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeToml {
    /// First endpoint station id.
    pub from: StationId,
    /// Second endpoint station id.
    pub to: StationId,
    /// Transport mode of the route.
    pub transport: Transport,
}
