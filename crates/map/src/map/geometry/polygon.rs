//! `Polygon` - a closed ring of points in map mm.

use crate::Segment;
use crate::map::blocks::SquareMillimeters;
use crate::map::geometry::line::Line;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use std::ops::Deref;

/// The miter cap for [`Polygon::offset`] corners - acute corners
/// may extend at most this many edge distances.
const MITER_CAP: f64 = 4.0;

/// A closed ring of points: footprints, block outlines, park
/// blobs - the last point connects back to the first. Derefs to
/// the `[Vec2]` slice so every `&[Vec2]` geometry helper keeps
/// working.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon(pub Vec<Vec2<Millimeters>>);

impl Polygon {
    /// A polygon from its vertex ring.
    pub fn new(points: Vec<Vec2<Millimeters>>) -> Self {
        Self(points)
    }

    /// Removes and returns the vertex at `index`.
    pub fn remove(&mut self, index: usize) -> Vec2<Millimeters> {
        self.0.remove(index)
    }

    /// The ring's edge segments - the last vertex connects back
    /// to the first.
    pub fn segments(&self) -> impl Iterator<Item = Segment<Millimeters>> + '_ {
        let points = &self.0;
        (0..points.len()).map(|i| Segment::new(points[i], points[(i + 1) % points.len()]))
    }

    /// The shoelace signed area in mm² (positive =
    /// counter-clockwise in the coordinate frame).
    pub fn signed_area(&self) -> SquareMillimeters {
        self.segments()
            .map(|edge| edge.from.cross::<SquareMillimeters>(&edge.to))
            .sum::<SquareMillimeters>()
            * 0.5
    }

    /// The ring's vertex centroid (uniform vertex average).
    pub fn centroid(&self) -> Vec2<Millimeters> {
        let count = self.0.len().max(1) as f64;
        Vec2::new(self.0.iter().map(|p| p.x).sum::<f64>() / count, self.0.iter().map(|p| p.y).sum::<f64>() / count)
    }

    /// Whether `point` lies strictly inside the ring (even-odd
    /// ray cast).
    pub fn contains(&self, point: &Vec2<Millimeters>) -> bool {
        let mut inside = false;
        for edge in self.segments() {
            let (a, b) = (edge.from, edge.to);
            let crosses_ray = (a.y > point.y) != (b.y > point.y);
            if crosses_ray && point.x < a.x + (b.x - a.x) * (point.y - a.y) / (b.y - a.y) {
                inside = !inside;
            }
        }
        inside
    }

    /// Whether `point` lies inside the ring or on its boundary -
    /// [`Self::contains`] plus a tolerance for facade points
    /// riding on an outline edge.
    pub fn contains_or_touches(&self, point: &Vec2<Millimeters>) -> bool {
        const EDGE_EPS: Millimeters = Millimeters::new(0.06);
        if self.contains(point) {
            return true;
        }
        self.segments().any(|edge| {
            edge.interior_distance(point).is_some_and(|d| d < EDGE_EPS) || point.distance(&edge.from) < EDGE_EPS || point.distance(&edge.to) < EDGE_EPS
        })
    }

    /// Whether the ring is simple - no two non-adjacent edges
    /// cross.
    pub fn is_simple(&self) -> bool {
        let edges: Vec<Segment<Millimeters>> = self.segments().collect();
        let n = edges.len();
        for i in 0..n {
            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }
                if edges[i].crosses(&edges[j]) {
                    return false;
                }
            }
        }
        true
    }

    /// Whether `segment` enters the ring: a strict edge crossing
    /// or an endpoint inside. Tangent grazes do not count.
    pub fn crossed(&self, segment: &Segment<Millimeters>) -> bool {
        self.segments().any(|edge| segment.crosses(&edge)) || self.contains(&segment.from) || self.contains(&segment.to)
    }

    /// Whether `other` overlaps the ring: an edge crossing or a
    /// vertex of either inside the other.
    pub fn overlaps(&self, other: &[Vec2<Millimeters>]) -> bool {
        let other = Polygon::new(other.to_vec());
        self.segments().any(|edge| other.segments().any(|other_edge| edge.crosses(&other_edge)))
            || self.iter().any(|p| other.contains(p))
            || other.iter().any(|p| self.contains(p))
    }

    /// Pulls each edge of the ring (positive signed area,
    /// interior left of each edge) `distances[i]` mm inward.
    /// Corner vertices are the intersections of the two offset
    /// edge lines, miter-capped. Not validated - the caller
    /// checks the result (area, containment).
    pub fn offset(&self, distances: &[f64]) -> Polygon {
        let n = self.0.len();
        let mut lines = Vec::with_capacity(n);
        for (i, edge) in self.segments().enumerate() {
            let direction = edge.to - edge.from;
            let unit = direction.normalized();
            // Left normal points into the polygon (positive area).
            lines.push(Line {
                origin: edge.from + unit.perpendicular() * distances[i],
                direction: unit,
            });
        }
        Polygon::new(
            (0..n)
                .map(|i| {
                    let previous = (i + n - 1) % n;
                    let (l1, l2) = (lines[previous], lines[i]);
                    let corner = self.0[i];
                    let distance = distances[previous].max(distances[i]);
                    // Near-parallel fallback: average the two
                    // normals - the bounded bisector inset.
                    let bisector = || {
                        let normal = l1.direction.perpendicular() * distances[previous] + l2.direction.perpendicular() * distances[i];
                        corner + normal * (distance / normal.magnitude().as_f64().max(1e-9))
                    };
                    let Some(inset) = l1.intersect(&l2) else {
                        return bisector();
                    };
                    let miter = inset - corner;
                    if miter.magnitude().as_f64() <= MITER_CAP * distance {
                        return inset;
                    }
                    // Overshot miter: acute corners legitimately run
                    // long in the bisector direction - cap them.
                    // Degenerate corners (near-180° vertices, e.g.
                    // a `via` station on a straight street) fly
                    // off sideways - there the bounded bisector
                    // inset is right.
                    let bis = bisector();
                    let miter_bisector = bis - corner;
                    let aligned = miter.dot::<f64>(&miter_bisector) / (miter.magnitude().as_f64() * miter_bisector.magnitude().as_f64()).max(1e-9);
                    if aligned > 0.5 {
                        corner + miter * (MITER_CAP * distance / miter.magnitude().as_f64())
                    } else {
                        bis
                    }
                })
                .collect(),
        )
    }

    /// The chord of the ring through `point` along `axis`: the
    /// segment between the nearest outline crossings bracketing
    /// `point` on the line `point + axis * t` - `None` when
    /// `point` sits outside.
    pub fn chord(&self, point: &Vec2<Millimeters>, axis: &Vec2<Millimeters>) -> Option<Segment<Millimeters>> {
        let epsilon = Millimeters::new(0.01);
        let (t_lo, t_hi) =
            self.segments()
                .filter_map(|edge| edge.param_on_line(point, axis))
                .fold((Millimeters::new(f64::MIN), Millimeters::INFINITY), |(lo, hi), t| {
                    (
                        if t < -epsilon { Millimeters::max(lo, t) } else { lo },
                        if t > epsilon { Millimeters::min(hi, t) } else { hi },
                    )
                });
        if !t_lo.as_f64().is_finite() || !t_hi.as_f64().is_finite() {
            return None;
        }
        Some(Segment::new(*point + *axis * t_lo.as_f64(), *point + *axis * t_hi.as_f64()))
    }

    /// The distance from `point` to the nearest ring edge (mm) -
    /// how far inside the polygon it sits; `deepest_point`
    /// maximizes it.
    pub fn clearance(&self, point: &Vec2<Millimeters>) -> Millimeters {
        self.segments().map(|edge| edge.distance(point)).fold(Millimeters::INFINITY, Millimeters::min)
    }

    /// The interior point with the largest clearance to the
    /// ring's edges (sampled on a 7x7 grid) - the spot a label
    /// sits best.
    pub fn deepest_point(&self) -> Option<Vec2<Millimeters>> {
        self.deepest(7, &[])
    }

    /// Like [`Self::deepest_point`], but skips candidates inside
    /// `avoid` polygons or within `AVOID_GAP` mm of their edges -
    /// so a district caption never lands on top of a park name.
    pub fn deepest_point_avoiding(&self, avoid: &[Polygon]) -> Option<Vec2<Millimeters>> {
        self.deepest(9, avoid)
    }

    /// The grid-sampled deepest interior point, optionally
    /// blocking zones the caption must dodge.
    fn deepest(&self, steps: usize, avoid: &[Polygon]) -> Option<Vec2<Millimeters>> {
        const AVOID_GAP: Millimeters = Millimeters::new(3.0);
        let blocked = |point: &Vec2<Millimeters>| {
            avoid
                .iter()
                .any(|polygon| polygon.contains(point) || polygon.segments().any(|edge| edge.interior_distance(point).is_some_and(|d| d < AVOID_GAP)))
        };
        let (min_x, max_x) = self.0.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(p.x), hi.max(p.x)));
        let (min_y, max_y) = self.0.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(p.y), hi.max(p.y)));

        let mut best: Option<(Millimeters, Vec2<Millimeters>)> = None;
        for row in 1..steps {
            for col in 1..steps {
                let point = Vec2::new(min_x + (max_x - min_x) * row as f64 / steps as f64, min_y + (max_y - min_y) * col as f64 / steps as f64);
                if !self.contains(&point) || blocked(&point) {
                    continue;
                }
                let score = self.clearance(&point);
                if best.is_none_or(|(best_score, _)| score > best_score) {
                    best = Some((score, point));
                }
            }
        }
        best.map(|(_, point)| point)
    }
}

impl<'a> IntoIterator for &'a Polygon {
    type Item = &'a Vec2<Millimeters>;
    type IntoIter = std::slice::Iter<'a, Vec2<Millimeters>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Polygon {
    type Target = [Vec2<Millimeters>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Vec2<Millimeters>> for Polygon {
    fn from_iter<I: IntoIterator<Item = Vec2<Millimeters>>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl From<Vec<Vec2<Millimeters>>> for Polygon {
    fn from(points: Vec<Vec2<Millimeters>>) -> Self {
        Self::new(points)
    }
}

impl From<Polygon> for Vec<Vec2<Millimeters>> {
    fn from(polygon: Polygon) -> Self {
        polygon.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 10mm square, counter-clockwise (positive signed area).
    fn square() -> Polygon {
        Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)])
    }

    #[test]
    fn segments_close_the_ring() {
        let segments: Vec<Segment<Millimeters>> = square().segments().collect();
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[3], Segment::new(Vec2::new(0.0, 10.0), Vec2::new(0.0, 0.0)));
    }

    #[test]
    fn signed_area_counts_ccw_positively() {
        assert_eq!(square().signed_area(), SquareMillimeters::new(100.0));
        let mut ring = square().0;
        ring.reverse();
        assert_eq!(Polygon::new(ring).signed_area(), SquareMillimeters::new(-100.0));
    }

    #[test]
    fn centroid_averages_the_vertices() {
        assert_eq!(square().centroid(), Vec2::new(5.0, 5.0));
    }

    #[test]
    fn contains_distinguishes_inside_from_outside() {
        assert!(square().contains(&Vec2::new(5.0, 5.0)));
        assert!(!square().contains(&Vec2::new(15.0, 5.0)));
        assert!(!square().contains(&Vec2::new(-1.0, 5.0)));
    }

    #[test]
    fn contains_or_touches_accepts_the_boundary() {
        assert!(square().contains_or_touches(&Vec2::new(0.0, 5.0)));
        assert!(square().contains_or_touches(&Vec2::new(10.0, 10.0)));
        assert!(!square().contains_or_touches(&Vec2::new(-1.0, 5.0)));
    }

    #[test]
    fn is_simple_rejects_a_bowtie() {
        let bowtie = Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(10.0, 0.0), Vec2::new(0.0, 10.0)]);
        assert!(square().is_simple());
        assert!(!bowtie.is_simple());
    }

    #[test]
    fn crossed_detects_entries_and_inside_segments() {
        let entering = Segment::new(Vec2::new(-5.0, 5.0), Vec2::new(5.0, 5.0));
        let inside = Segment::new(Vec2::new(2.0, 5.0), Vec2::new(8.0, 5.0));
        let outside = Segment::new(Vec2::new(-5.0, 20.0), Vec2::new(15.0, 20.0));
        assert!(square().crossed(&entering));
        assert!(square().crossed(&inside), "a segment fully inside crosses");
        assert!(!square().crossed(&outside));
    }

    #[test]
    fn overlaps_detects_intersection_and_containment() {
        let overlapping = vec![Vec2::new(5.0, 5.0), Vec2::new(15.0, 5.0), Vec2::new(15.0, 15.0), Vec2::new(5.0, 15.0)];
        let nested = vec![Vec2::new(2.0, 2.0), Vec2::new(4.0, 2.0), Vec2::new(4.0, 4.0), Vec2::new(2.0, 4.0)];
        let far = vec![Vec2::new(20.0, 20.0), Vec2::new(30.0, 20.0), Vec2::new(30.0, 30.0), Vec2::new(20.0, 30.0)];
        assert!(square().overlaps(&overlapping));
        assert!(square().overlaps(&nested));
        assert!(!square().overlaps(&far));
    }

    #[test]
    fn remove_extracts_the_vertex() {
        let mut polygon = square();
        assert_eq!(polygon.remove(1), Vec2::new(10.0, 0.0));
        assert_eq!(polygon.len(), 3);
    }

    #[test]
    fn offset_shrinks_the_ring_inward() {
        let inset = square().offset(&[1.0, 1.0, 1.0, 1.0]);
        assert_eq!(inset.len(), 4);
        assert!(inset.contains(&Vec2::new(5.0, 5.0)));
        assert!(!inset.contains(&Vec2::new(0.5, 5.0)), "the 1mm band is cut off");
        assert_eq!(inset.signed_area(), SquareMillimeters::new(64.0));
    }

    #[test]
    fn clearance_measures_the_edge_distance() {
        assert_eq!(square().clearance(&Vec2::new(5.0, 5.0)), Millimeters::new(5.0));
        assert!(square().clearance(&Vec2::new(1.0, 5.0)) < Millimeters::new(5.0));
    }

    #[test]
    fn deepest_point_sits_inside_with_margin() {
        let point = square().deepest_point().unwrap_or_else(|| panic!("a square has an interior"));
        assert!(square().contains(&point));
        assert!(square().clearance(&point) > Millimeters::new(2.0));
    }

    #[test]
    fn deepest_point_avoids_the_blocked_zone() {
        let avoid = vec![Polygon::new(vec![
            Vec2::new(4.0, 4.0),
            Vec2::new(6.0, 4.0),
            Vec2::new(6.0, 6.0),
            Vec2::new(4.0, 6.0),
        ])];
        let point = square().deepest_point_avoiding(&avoid).unwrap_or_else(|| panic!("room remains"));
        assert!(square().contains(&point));
        assert!(!avoid[0].contains(&point), "the caption dodges the park");
    }
}
