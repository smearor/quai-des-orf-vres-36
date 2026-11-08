//! `CutSegment` - a map feature's line segment plus its cut kind.

use crate::Segment;
use crate::map::blocks::BlockCut;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// One raw cut line of the block subdivision: a street hop, a river
/// course piece or a border edge.
pub struct CutSegment {
    /// Start position in map mm.
    pub from: Vec2<Millimeters>,
    /// End position in map mm.
    pub to: Vec2<Millimeters>,
    /// The feature this line comes from.
    pub cut: BlockCut,
}

impl CutSegment {
    /// A cut segment adopting `segment`'s geometry, tagged `cut`.
    pub fn of(segment: Segment<Millimeters>, cut: BlockCut) -> Self {
        Self {
            from: segment.from,
            to: segment.to,
            cut,
        }
    }
}

impl From<&CutSegment> for Segment<Millimeters> {
    fn from(cut: &CutSegment) -> Self {
        Segment::new(cut.from, cut.to)
    }
}
