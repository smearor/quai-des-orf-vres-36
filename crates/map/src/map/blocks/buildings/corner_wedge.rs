//! `CornerWedge` - the flatiron wedge at a ring vertex: the two
//! edge frames plus the rolled dimensions.

use crate::map::Polyline;
use crate::map::blocks::Depth;
use crate::map::blocks::buildings::CORNER_ARC_STEPS;
use crate::map::blocks::buildings::arc_sweep;
use crate::map::blocks::buildings::corner_nose::CornerNose;
use crate::map::blocks::buildings::edge_frame::EdgeFrame;
use crate::map::geometry::Line;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// The flatiron wedge at a ring vertex: the corner position, the
/// edge frames entering (`prev`) and leaving (`next`) it, and
/// the rolled facade width and depth.
pub(super) struct CornerWedge<'a> {
    /// The vertex position in map mm.
    pub corner: Vec2<Millimeters>,
    /// The edge frame entering the vertex.
    pub prev: &'a EdgeFrame,
    /// The edge frame leaving the vertex.
    pub next: &'a EdgeFrame,
    /// The facade frontage (`units * house width`).
    pub frontage: Millimeters,
    /// The house depth.
    pub depth: Depth,
}

impl CornerWedge<'_> {
    /// The tangent nose arc bulging toward the vertex plus the
    /// facade lengths consumed on both edges (the facade may not
    /// run past half a neighbor edge).
    pub fn nose(&self, interior: f64) -> CornerNose {
        let depth = Millimeters::new(self.depth.as_f64());
        let half = (interior / 2.0).max(0.05);
        let radius = (depth * 0.9).min(self.frontage * 0.5).max(Millimeters::new(0.8));
        let tangent_len = (radius / half.tan()).min(self.prev.length.min(self.next.length) * 0.5);
        let center_dist = radius / half.sin();
        let (bisector_x, bisector_y) = (self.next.direction.x - self.prev.direction.x, self.next.direction.y - self.prev.direction.y);
        let bisector_len = bisector_x.hypot(bisector_y).max(1e-9);
        let bisector = Vec2::<Millimeters>::new(bisector_x / bisector_len, bisector_y / bisector_len);
        let center = self.corner + bisector * center_dist.as_f64();
        let nose_prev = self.corner - self.prev.direction * tangent_len.as_f64();
        let nose_next = self.corner + self.next.direction * tangent_len.as_f64();
        let facade_prev = self.frontage.max(tangent_len).min(self.prev.length * 0.5);
        let facade_next = self.frontage.max(tangent_len).min(self.next.length * 0.5);
        let front_prev = self.corner - self.prev.direction * facade_prev.as_f64();
        let front_next = self.corner + self.next.direction * facade_next.as_f64();
        // The arc sweeps the circle part facing the vertex.
        let start = (nose_prev.y - center.y).atan2(nose_prev.x - center.x);
        let end = (nose_next.y - center.y).atan2(nose_next.x - center.x);
        let vertex_angle = (self.corner.y - center.y).atan2(self.corner.x - center.x);
        let sweep = arc_sweep(start, end, vertex_angle);
        let mut points = vec![nose_prev];
        for step in 1..CORNER_ARC_STEPS.as_usize() {
            let angle = start + sweep * step as f64 / CORNER_ARC_STEPS.as_f64();
            points.push(Vec2::new(center.x + angle.cos() * radius.as_f64(), center.y + angle.sin() * radius.as_f64()));
        }
        points.push(nose_next);
        CornerNose {
            front_prev,
            front_next,
            points: Polyline::new(points),
            facade_prev,
            facade_next,
        }
    }

    /// The back wall behind `front_prev`/`front_next`: straight
    /// sides at `depth` plus the offset inner corner where they
    /// meet (miter-capped on very acute corners).
    pub fn back_wall(&self, front_prev: Vec2<Millimeters>, front_next: Vec2<Millimeters>) -> [Vec2<Millimeters>; 3] {
        let depth = self.depth.as_f64();
        let back_next = front_next + self.next.inward * depth;
        let back_prev = front_prev + self.prev.inward * depth;
        let inner = Line {
            origin: self.corner + self.prev.inward * depth,
            direction: self.prev.direction,
        }
        .intersect(&Line {
            origin: self.corner + self.next.inward * depth,
            direction: self.next.direction,
        })
        .map(|point| {
            // Miter cap for very acute corners.
            let (miter_x, miter_y) = (point.x - self.corner.x, point.y - self.corner.y);
            let miter_cap = depth * 4.0;
            let miter_len = miter_x.hypot(miter_y);
            if miter_len > miter_cap {
                self.corner + Vec2::new(miter_x, miter_y) * (miter_cap / miter_len)
            } else {
                point
            }
        })
        .unwrap_or_else(|| (back_prev + back_next) / 2.0);
        [back_next, inner, back_prev]
    }
}
