//! `EdgeFrame` - one edge's direction data for house placement.

use crate::map::Polygon;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

use crate::map::blocks::Angle;

/// One edge's direction data for house placement.
pub(super) struct EdgeFrame {
    /// Unit vector along the edge.
    pub(super) direction: Vec2<Millimeters>,
    /// Left unit normal, pointing into the block.
    pub(super) inward: Vec2<Millimeters>,
    /// Edge length.
    pub(super) length: Millimeters,
}

impl EdgeFrame {
    /// The frame of one edge `from -> to`: direction and inward
    /// (left) unit normal plus the edge's length.
    pub(super) fn new(from: Vec2<Millimeters>, to: Vec2<Millimeters>) -> Self {
        let delta = to - from;
        let length = delta.magnitude().max(Millimeters::new(1e-9));
        let direction = delta.normalized();
        Self {
            direction,
            inward: direction.perpendicular(),
            length,
        }
    }

    /// The edge frames of a ring polygon (one per edge).
    pub(super) fn of_ring(ring: &Polygon) -> Vec<Self> {
        ring.segments().map(|edge| Self::new(edge.from, edge.to)).collect()
    }

    /// The interior angle at vertex `i` of a positive-area
    /// polygon's ring (sharp corners < `CORNER_MAX_ANGLE`).
    pub(super) fn interior_angle(frames: &[EdgeFrame], i: usize) -> Angle {
        let n = frames.len();
        Angle::interior(frames[(i + n - 1) % n].direction, frames[i].direction)
    }
}
