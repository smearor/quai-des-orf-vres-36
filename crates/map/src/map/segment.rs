//! `Segment` - a straight segment between two positions.

use crate::map::Polygon;
use crate::map::Polyline;
use crate::map::Projection;
use crate::map::blocks::Angle;
use std::f64::consts::PI;
use tangible_objects_model::Millimeters;
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

    /// The segment's direction as an angle (atan2 of the
    /// from→to vector).
    pub fn direction(&self) -> Angle {
        Angle::new((self.to.y - self.from.y).atan2(self.to.x - self.from.x))
    }

    /// Strict crossing - `true` when the interiors of both segments
    /// intersect (shared endpoints are junctions, not crossings).
    pub fn crosses(&self, other: &Self) -> bool {
        let delta = |a: Vec2<U>, b: Vec2<U>| Vec2::<U>::new(b.x - a.x, b.y - a.y);
        let orient = |p: Vec2<U>, q: Vec2<U>, r: Vec2<U>| delta(p, q).cross::<f64>(&delta(p, r));
        orient(self.from, self.to, other.from) * orient(self.from, self.to, other.to) < 0.0
            && orient(other.from, other.to, self.from) * orient(other.from, other.to, self.to) < 0.0
    }

    /// The unclamped projection of `point` onto the segment's
    /// line - `None` only when the segment is degenerate.
    fn project(&self, point: &Vec2<U>) -> Option<Projection<U>> {
        let direction = self.to - self.from;
        let len_sq = direction.dot::<f64>(&direction);
        if len_sq == 0.0 {
            return None;
        }
        let fraction = (*point - self.from).dot::<f64>(&direction) / len_sq;
        Some(Projection {
            fraction,
            point: self.from + direction * fraction,
        })
    }

    /// The projection of `point` onto the segment interior -
    /// `None` when the projection falls on an endpoint or
    /// outside the segment, or the segment is degenerate. Unlike
    /// `interior_distance` the endpoints are excluded (a snapped
    /// endpoint is no cut point).
    pub fn interior_projection(&self, point: &Vec2<U>) -> Option<Projection<U>> {
        let projection = self.project(point)?;
        (1e-6..=1.0 - 1e-6).contains(&projection.fraction).then_some(projection)
    }

    /// Perpendicular distance of `point` to the segment when the
    /// closest point lies between the endpoints - `None` when the
    /// projection falls beyond either end or the segment is
    /// degenerate.
    pub fn interior_distance(&self, point: &Vec2<U>) -> Option<U>
    where
        U: From<f64>,
    {
        let projection = self.project(point)?;
        (0.0..=1.0).contains(&projection.fraction).then(|| point.distance(&projection.point))
    }

    /// The nearest point on the segment to `point` plus its
    /// `fraction` along `from -> to` (clamped to `[0, 1]`) -
    /// `None` for a degenerate segment.
    pub fn nearest(&self, point: &Vec2<U>) -> Option<Projection<U>> {
        let fraction = self.project(point)?.fraction.clamp(0.0, 1.0);
        Some(Projection {
            fraction,
            point: self.from + (self.to - self.from) * fraction,
        })
    }

    /// The distance from `point` to the segment - the
    /// perpendicular distance when the projection lands inside,
    /// the nearer endpoint distance otherwise.
    pub fn distance(&self, point: &Vec2<U>) -> U
    where
        U: From<f64>,
    {
        self.nearest(point)
            .map_or_else(|| point.distance(&self.from), |nearest| point.distance(&nearest.point))
    }

    /// The point where the two segments strictly cross - `None`
    /// when they do not (a mere touch counts as a junction, not
    /// a crossing).
    pub fn crossing_point(&self, other: &Self) -> Option<Vec2<U>> {
        self.crosses(other).then(|| self.intersection(other)).flatten()
    }

    /// The parameter `t` where the line `anchor + axis * t` meets
    /// this segment - `None` when they are parallel or the meeting
    /// lies off the segment.
    pub fn param_on_line(&self, anchor: &Vec2<U>, axis: &Vec2<U>) -> Option<U>
    where
        U: From<f64>,
    {
        let edge_direction = self.to - self.from;
        let denom = axis.cross::<f64>(&edge_direction);
        if denom.abs() < 1e-9 {
            return None;
        }
        let offset = self.from - *anchor;
        let s = axis.cross::<f64>(&offset) / -denom;
        if !(0.0..=1.0).contains(&s) {
            return None;
        }
        Some(U::from(offset.cross::<f64>(&edge_direction) / denom))
    }

    /// The point where the two segments meet - `None` when they are
    /// parallel, degenerate or the meeting lies outside either
    /// segment.
    pub fn intersection(&self, other: &Self) -> Option<Vec2<U>> {
        let direction = Vec2::<U>::new(self.to.x - self.from.x, self.to.y - self.from.y);
        let other_direction = Vec2::<U>::new(other.to.x - other.from.x, other.to.y - other.from.y);
        let denom = direction.cross::<f64>(&other_direction);
        if denom == 0.0 {
            return None;
        }
        let offset = Vec2::<U>::new(other.from.x - self.from.x, other.from.y - self.from.y);
        let t = offset.cross::<f64>(&other_direction) / denom;
        let u = offset.cross::<f64>(&direction) / denom;
        if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
            return None;
        }
        Some(Vec2::new(self.from.x + t * direction.x, self.from.y + t * direction.y))
    }
}

impl Segment<Millimeters> {
    /// The sine arc from `from` to `to`, bowed `amplitude` to the
    /// left of the direction at its peak (negative bows right) -
    /// `steps + 1` points including the endpoints.
    pub fn arc(&self, steps: usize, amplitude: Millimeters) -> Polyline {
        let direction = self.to - self.from;
        let normal = direction.normalized().perpendicular();
        (0..=steps)
            .map(|step| {
                let t = step as f64 / steps as f64;
                let bend = (PI * t).sin() * amplitude.as_f64();
                self.from + direction * t + normal * bend
            })
            .collect()
    }

    /// A rectangle corridor `half_width` mm to each side of the
    /// segment - the obstacle shape tunnels and decorative
    /// streets block houses with.
    pub fn corridor(&self, half_width: Millimeters) -> Polygon {
        let direction = self.to - self.from;
        let normal = direction.normalized().perpendicular() * half_width.as_f64();
        Polygon::new(vec![self.from + normal, self.to + normal, self.to - normal, self.from - normal])
    }
}
