//! `Neighbors` - the adjacency list of a station.

use crate::LineName;
use crate::Transport;
use crate::station::Neighbor;
use crate::station::StationId;
use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use std::ops::DerefMut;

/// The stations adjacent to a station plus the transports
/// reaching them. Derefs to the `Vec<Neighbor>`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Neighbors(pub Vec<Neighbor>);

/// The empty adjacency - unknown stations read as this.
pub static EMPTY: Neighbors = Neighbors(Vec::new());

impl Neighbors {
    /// The metro lines serving the station, deduplicated.
    pub fn metro_lines(&self) -> Vec<LineName> {
        let mut lines: Vec<LineName> = Vec::new();
        for neighbor in self.0.iter() {
            let Some(line) = &neighbor.line else { continue };
            if !lines.contains(line) {
                lines.push(line.clone());
            }
        }
        lines
    }

    /// The transport modes of the station's edges, deduplicated.
    pub fn transports(&self) -> Vec<Transport> {
        let mut transports: Vec<Transport> = Vec::new();
        for neighbor in self.0.iter() {
            if !transports.contains(&neighbor.transport) {
                transports.push(neighbor.transport);
            }
        }
        transports
    }

    /// The adjacent station ids.
    pub fn stations(&self) -> impl Iterator<Item = StationId> + '_ {
        self.0.iter().map(|neighbor| neighbor.station)
    }

    /// The adjacent stations reachable via `transport`.
    pub fn stations_by(&self, transport: Transport) -> impl Iterator<Item = StationId> + '_ {
        self.stations_where(move |serves| serves == transport)
    }

    /// The adjacent stations whose transport `pred` accepts.
    pub fn stations_where<'a>(&'a self, mut pred: impl FnMut(Transport) -> bool + 'a) -> impl Iterator<Item = StationId> + 'a {
        self.0.iter().filter_map(move |neighbor| pred(neighbor.transport).then_some(neighbor.station))
    }
}

impl Deref for Neighbors {
    type Target = Vec<Neighbor>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Neighbors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
