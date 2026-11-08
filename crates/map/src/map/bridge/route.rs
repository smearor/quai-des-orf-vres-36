//! `RoadRoute` - a route polyline plus its per-segment hop pairs.

use crate::Segment;
use crate::map::Polyline;
use crate::map::blocks::BlockCut;
use crate::map::blocks::CutSegment;
use crate::station::StationPair;

/// A road route's drawn polyline plus, for every segment, the
/// station pair the segment's outgoing hop connects - the
/// parallel-transports overlay shifts each segment by its hop's
/// lane offset.
#[derive(Debug, Clone, PartialEq)]
pub struct RoadRoute {
    /// The route polyline in map mm.
    pub polyline: Polyline,
    /// `hops[segment]` = the station pair that segment's hop
    /// connects.
    pub hops: Vec<StationPair>,
}

impl RoadRoute {
    /// The street cut segments the route contributes to the
    /// block layout: one per polyline leg, tagged with the leg's
    /// station pair (zero-length legs skipped).
    pub fn street_cuts(&self) -> Vec<CutSegment> {
        self.polyline
            .windows(2)
            .enumerate()
            .filter(|(_, hop)| hop[0] != hop[1])
            .map(|(index, hop)| CutSegment::of(Segment::new(hop[0], hop[1]), BlockCut::Street(self.hops[index])))
            .collect()
    }
}
