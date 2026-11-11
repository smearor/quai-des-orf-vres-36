//! `Projection` - a point's orthogonal projection onto a
//! segment.

use smearor_commons_geometry::Vec2;

/// The orthogonal projection of a point onto a segment interior:
/// where along the segment it lands and at which point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection<U> {
    /// The fraction along the segment in (0, 1): the projected
    /// point is `from + fraction * (to - from)`. Endpoints are
    /// excluded - a snapped endpoint is no cut point.
    pub fraction: f64,
    /// The projected point on the segment, in unit `U`.
    pub point: Vec2<U>,
}
