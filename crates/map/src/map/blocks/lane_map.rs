//! `LaneMap` - which lanes occupy each street hop.

use crate::LineName;
use crate::Transport;
use crate::map::blocks::HopLanes;
use crate::station::StationPair;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

/// The lanes per street hop, recorded while the edges' drawn
/// paths are folded into the block layout. Derefs to the
/// `HashMap`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LaneMap(pub HashMap<StationPair, HopLanes>);

impl LaneMap {
    /// Records one drawn edge hop: taxi and bus set their flag,
    /// a lined metro joins the line set, an unlined metro counts
    /// up.
    pub fn record(&mut self, hop: StationPair, transport: Transport, line: Option<LineName>) {
        let entry = self.0.entry(hop).or_default();
        match transport {
            Transport::Taxi => entry.taxi = true,
            Transport::Bus => entry.bus = true,
            Transport::Metro => match line {
                Some(line) => {
                    entry.metro_lines.insert(line);
                }
                None => entry.unlined_metro += 1,
            },
            _ => {}
        }
    }

    /// The lanes a hop carries under `overlay` - at least one.
    pub fn lane_count(&self, hop: &StationPair, overlay: bool) -> usize {
        self.0.get(hop).map(|lanes| lanes.lanes(overlay).max(1)).unwrap_or(1)
    }
}

impl Deref for LaneMap {
    type Target = HashMap<StationPair, HopLanes>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LaneMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
