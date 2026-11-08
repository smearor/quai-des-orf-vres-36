//! `BlockLayout` - the map's city-block division, computed once.

use crate::map::WATER_HALF_WIDTH;
use crate::map::blocks::BlockCut;
use crate::map::blocks::BlockFaces;
use crate::map::blocks::LaneMap;
use tangible_objects_model::Millimeters;

/// The city-block division of the map: the free faces enclosed by
/// streets, river and border plus the lane occupancy of each street
/// hop - everything the renderer needs to draw block outlines with
/// the right setbacks, independent of the current view mode.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockLayout {
    /// Faces of the planar subdivision (positive-area outlines).
    pub faces: BlockFaces,
    /// Lanes per street hop - only hops present as a `Street` cut.
    pub lanes: LaneMap,
}

/// Clearance between a block outline and its bounding feature.
const BLOCK_GAP: Millimeters = Millimeters::new(2.0);
/// Half the corridor of a single drawn lane.
const ROAD_HALF: Millimeters = Millimeters::new(0.9);
/// Corridor growth per extra parallel lane (half lane step).
const LANE_STEP: Millimeters = Millimeters::new(2.6);

impl BlockLayout {
    /// The setback a block's inset keeps from `cut`: streets
    /// reserve corridor for the lanes drawn over them - one
    /// centered lane in standard view, every transport (plus each
    /// metro line under the metro overlay) in parallel view -
    /// water keeps its channel, the border only half the gap.
    pub fn setback(&self, cut: &BlockCut, parallel: bool, metro_overlay: bool) -> Millimeters {
        match cut {
            BlockCut::Street(pair) => {
                let lanes = if parallel { self.lanes.lane_count(pair, metro_overlay) } else { 1 };
                BLOCK_GAP + ROAD_HALF + LANE_STEP * (lanes.saturating_sub(1) as f64 / 2.0)
            }
            BlockCut::Water => BLOCK_GAP + WATER_HALF_WIDTH,
            BlockCut::Border => BLOCK_GAP * 0.5,
        }
    }
}
