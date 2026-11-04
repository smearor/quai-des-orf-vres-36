//! `Segment` - a straight segment between two positions.

use tangible_objects_model::Vec2;

/// A straight segment in unit `U` (`Vec2<Millimeters>` on the map,
/// `Vec2<Pixels>` on screen) - the geometry an edge draws between
/// its two endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment<U> {
    /// Segment start position.
    pub from: Vec2<U>,
    /// Segment end position.
    pub to: Vec2<U>,
}

impl<U> Segment<U> {
    /// A segment from `from` to `to`.
    pub const fn new(from: Vec2<U>, to: Vec2<U>) -> Self {
        Self { from, to }
    }
}

impl<U: Copy> Segment<U> {
    /// The segment's center point.
    pub fn midpoint(&self) -> Vec2<U> {
        Vec2::new((self.from.x + self.to.x) / 2.0, (self.from.y + self.to.y) / 2.0)
    }

    /// Strict crossing - `true` when the interiors of both segments
    /// intersect (shared endpoints are junctions, not crossings).
    pub fn crosses(&self, other: &Self) -> bool {
        let orient = |p: Vec2<U>, q: Vec2<U>, r: Vec2<U>| (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x);
        orient(self.from, self.to, other.from) * orient(self.from, self.to, other.to) < 0.0
            && orient(other.from, other.to, self.from) * orient(other.from, other.to, self.to) < 0.0
    }

    /// Perpendicular distance of `point` to the segment when the
    /// closest point lies between the endpoints - `None` when the
    /// projection falls beyond either end or the segment is
    /// degenerate.
    pub fn interior_distance(&self, point: &Vec2<U>) -> Option<f64> {
        let (vx, vy) = (self.to.x - self.from.x, self.to.y - self.from.y);
        let len_sq = vx * vx + vy * vy;
        if len_sq == 0.0 {
            return None;
        }
        let t = ((point.x - self.from.x) * vx + (point.y - self.from.y) * vy) / len_sq;
        if !(0.0..=1.0).contains(&t) {
            return None;
        }
        Some((point.x - (self.from.x + t * vx)).hypot(point.y - (self.from.y + t * vy)))
    }

    /// The point where the two segments meet - `None` when they are
    /// parallel, degenerate or the meeting lies outside either
    /// segment.
    pub fn intersection(&self, other: &Self) -> Option<Vec2<U>> {
        let (vx, vy) = (self.to.x - self.from.x, self.to.y - self.from.y);
        let (wx, wy) = (other.to.x - other.from.x, other.to.y - other.from.y);
        let denom = vx * wy - vy * wx;
        if denom == 0.0 {
            return None;
        }
        let (ox, oy) = (other.from.x - self.from.x, other.from.y - self.from.y);
        let t = (ox * wy - oy * wx) / denom;
        let u = (ox * vy - oy * vx) / denom;
        if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
            return None;
        }
        Some(Vec2::new(self.from.x + t * vx, self.from.y + t * vy))
    }
}
