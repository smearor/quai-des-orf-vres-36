//! `Stations` - the map's station set indexed by station number.

use crate::LineName;
use crate::Transport;
use crate::station::NearestStation;
use crate::station::Neighbors;
use crate::station::Station;
use crate::station::StationId;
use crate::station::StationKind;
use crate::station::neighbors::EMPTY as EMPTY_NEIGHBORS;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

/// Stations indexed by their station number (ordered by id).
/// Derefs to the `BTreeMap`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stations(pub BTreeMap<StationId, Station>);

impl Stations {
    /// The transport modes served at each station, deduplicated.
    pub fn served_transports(&self) -> HashMap<StationId, Vec<Transport>> {
        self.0.iter().map(|(id, station)| (*id, station.neighbors.transports())).collect()
    }

    /// The metro lines served at each station, deduplicated -
    /// `hidden` lines dropped (closed overlays draw no fill).
    pub fn served_lines(&self, hidden: &BTreeSet<LineName>) -> HashMap<StationId, Vec<LineName>> {
        self.0
            .iter()
            .map(|(id, station)| (*id, station.neighbors.metro_lines().into_iter().filter(|line| !hidden.contains(line)).collect()))
            .collect()
    }

    /// The station nearest to `pos`, with its distance - `None`
    /// on an empty map. Linear scan - the map is small enough
    /// for this to be cheap.
    pub fn nearest(&self, pos: Vec2<Millimeters>) -> Option<NearestStation> {
        self.0
            .values()
            .map(|station| NearestStation {
                station: station.id,
                distance: pos.distance(&station.pos),
            })
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }

    /// Stations carrying the given kind flag (start, batobus,
    /// bridge, ...).
    pub fn of_kind(&self, kind: StationKind) -> impl Iterator<Item = &Station> {
        self.0.values().filter(move |station| station.kinds.contains(&kind))
    }

    /// The positions of the stations carrying `kind`.
    pub fn positions_of_kind(&self, kind: StationKind) -> impl Iterator<Item = Vec2<Millimeters>> + '_ {
        self.of_kind(kind).map(|station| station.pos)
    }

    /// The straight-line distance between two stations in map
    /// mm - `None` when either id is unknown.
    pub fn distance(&self, from: StationId, to: StationId) -> Option<Millimeters> {
        self.0.get(&from).zip(self.0.get(&to)).map(|(a, b)| a.distance(b))
    }

    /// Neighbors of a station with their transport modes - the
    /// empty adjacency for unknown ids.
    pub fn neighbors(&self, id: StationId) -> &Neighbors {
        self.0.get(&id).map_or(&EMPTY_NEIGHBORS, |station| &station.neighbors)
    }
}

impl Deref for Stations {
    type Target = BTreeMap<StationId, Station>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Stations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
