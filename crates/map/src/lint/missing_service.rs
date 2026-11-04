//! `MissingService` - which service a station lacks (Q5).

use serde::Deserialize;
use serde::Serialize;

/// Which service a station is missing (Q5: metro stations need
/// both taxi and bus).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MissingService {
    /// No taxi edge touches the station.
    Taxi,
    /// No bus edge touches the station.
    Bus,
    /// Neither taxi nor bus edges touch the station.
    Both,
}
