//! `Polyline` - an open line of points in map mm.

use crate::Segment;
use crate::map::Projection;
use serde::Deserialize;
use serde::Serialize;
use serde::de::Error as _;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use std::ops::Deref;

/// An open line of points: river courses, streets, decorative
/// chords, edge routes - the last point does not connect back to
/// the first. Derefs to the `[Vec2]` slice so every `&[Vec2]`
/// geometry helper keeps working.
///
/// Serializes flat like upstream `flat_points`: `[x0, y0, x1, y1, ...]`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Polyline(pub Vec<Vec2<Millimeters>>);

impl Polyline {
    /// A polyline from its points.
    pub fn new(points: Vec<Vec2<Millimeters>>) -> Self {
        Self(points)
    }

    /// The polyline's points.
    pub fn points(&self) -> &[Vec2<Millimeters>] {
        &self.0
    }

    /// The axis parameter `t` where the line `center + axis * t`
    /// meets this polyline - `None` when they do not intersect
    /// within `reach`.
    pub fn axis_param(&self, center: Vec2<Millimeters>, axis: Vec2<Millimeters>, reach: Millimeters) -> Option<Millimeters> {
        let half = reach.as_f64();
        let line = Segment::new(center - axis * half, center + axis * half);
        self.segments()
            .find_map(|segment| line.intersection(&segment).map(|point| Millimeters::new((point - center).dot::<f64>(&axis))))
    }

    /// The polyline smoothed `rounds` times except the points
    /// within `within` of `anchors`, which stay pinned.
    pub fn smoothed_anchored(&self, anchors: &[Vec2<Millimeters>], within: Millimeters, rounds: usize) -> Self {
        self.smoothed(rounds, &self.indices_within(anchors, within))
    }

    /// The indices of points lying within `within` of any
    /// `positions` - e.g. the waypoints pinned through a
    /// `smoothed` call.
    pub fn indices_within(&self, positions: &[Vec2<Millimeters>], within: Millimeters) -> Vec<usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, point)| positions.iter().any(|pos| pos.distance(point) < within))
            .map(|(index, _)| index)
            .collect()
    }

    /// The arc length along the polyline up to the line parameter
    /// `at`: `at`'s integer part is a segment index, its fraction
    /// the position along that segment. `Millimeters::ZERO` on
    /// fewer than two points.
    pub fn arc_length(&self, at: f64) -> Millimeters {
        if self.0.len() < 2 {
            return Millimeters::ZERO;
        }
        let index = (at.max(0.0).floor() as usize).min(self.0.len() - 1);
        let full = self
            .0
            .windows(2)
            .take(index)
            .fold(Millimeters::ZERO, |arc, pair| arc + pair[0].distance(&pair[1]));
        let (a, b) = (self.0[index], self.0[(index + 1).min(self.0.len() - 1)]);
        full + a.distance(&b) * at.max(0.0).fract()
    }

    /// The polyline shifted `distance` mm to the left of its
    /// direction: bend vertices average the adjacent segment
    /// normals, so the offset stays near-constant without miter
    /// spikes.
    pub fn offset(&self, distance: Millimeters) -> Self {
        let normal = |from: Vec2<Millimeters>, to: Vec2<Millimeters>| {
            let perpendicular = (to - from).normalized().perpendicular();
            (perpendicular.x, perpendicular.y)
        };
        let points: Vec<Vec2<Millimeters>> = self
            .0
            .iter()
            .enumerate()
            .map(|(index, point)| {
                let (mut normal_x, mut normal_y) = (0.0, 0.0);
                if index > 0 {
                    let (x, y) = normal(self.0[index - 1], *point);
                    normal_x += x;
                    normal_y += y;
                }
                if index + 1 < self.0.len() {
                    let (x, y) = normal(*point, self.0[index + 1]);
                    normal_x += x;
                    normal_y += y;
                }
                let norm = normal_x.hypot(normal_y).max(1e-6);
                Vec2::new(point.x + normal_x / norm * distance.as_f64(), point.y + normal_y / norm * distance.as_f64())
            })
            .collect();
        Self::new(points)
    }

    /// The segment index and projection nearest to `point` -
    /// `None` for an empty polyline.
    pub fn nearest_segment(&self, point: &Vec2<Millimeters>) -> Option<(usize, Projection<Millimeters>)> {
        self.segments()
            .enumerate()
            .filter_map(|(index, segment)| segment.nearest(point).map(|projection| (index, projection)))
            .min_by(|(_, a), (_, b)| point.distance(&a.point).total_cmp(&point.distance(&b.point)))
    }

    /// Whether `other` strictly crosses the polyline.
    pub fn crosses(&self, other: &Polyline) -> bool {
        self.segments().any(|a| other.segments().any(|b| a.crosses(&b)))
    }

    /// The polyline's point-to-point segments.
    pub fn segments(&self) -> impl Iterator<Item = Segment<Millimeters>> + '_ {
        self.0.windows(2).map(|pair| Segment::new(pair[0], pair[1]))
    }

    /// Corner-cutting subdivision (Chaikin): each round replaces
    /// every segment by points at 1/4 and 3/4, rounding corners
    /// and roughly doubling the point count - denser spline
    /// checkpoints and no miter spikes at sharp river bends.
    /// `pinned` lists point indices (of the original polyline)
    /// that stay on the course exactly - bridge stations sit on
    /// those vertices, so the water keeps flowing right under
    /// their markers.
    pub fn smoothed(&self, rounds: usize, pinned: &[usize]) -> Self {
        let mut flags: Vec<bool> = self.0.iter().enumerate().map(|(index, _)| pinned.contains(&index)).collect();
        let mut current = self.0.clone();
        for _ in 0..rounds {
            if current.len() < 3 {
                break;
            }
            let mut next = Vec::with_capacity(current.len() * 2);
            let mut next_flags = Vec::with_capacity(flags.len() * 2);
            next.push(current[0]);
            next_flags.push(flags[0]);
            for (index, window) in current.windows(2).enumerate() {
                let (from, to) = (window[0], window[1]);
                next.push(Vec2::new(from.x * 0.75 + to.x * 0.25, from.y * 0.75 + to.y * 0.25));
                next.push(Vec2::new(from.x * 0.25 + to.x * 0.75, from.y * 0.25 + to.y * 0.75));
                next_flags.extend_from_slice(&[false, false]);
                if flags.get(index + 1).copied().unwrap_or(false) {
                    next.push(to);
                    next_flags.push(true);
                }
            }
            if let (Some(last), Some(flag)) = (current.last(), flags.last()) {
                next.push(*last);
                next_flags.push(*flag);
            }
            (current, flags) = (next, next_flags);
        }
        Self::new(current)
    }
}

impl<'a> IntoIterator for &'a Polyline {
    type Item = &'a Vec2<Millimeters>;
    type IntoIter = std::slice::Iter<'a, Vec2<Millimeters>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Polyline {
    type Target = [Vec2<Millimeters>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Vec2<Millimeters>> for Polyline {
    fn from_iter<I: IntoIterator<Item = Vec2<Millimeters>>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl From<Vec<Vec2<Millimeters>>> for Polyline {
    fn from(points: Vec<Vec2<Millimeters>>) -> Self {
        Self::new(points)
    }
}

impl From<Polyline> for Vec<Vec2<Millimeters>> {
    fn from(polyline: Polyline) -> Self {
        polyline.0
    }
}

impl Serialize for Polyline {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut flat = Vec::with_capacity(self.0.len() * 2);
        for p in &self.0 {
            flat.push(p.x);
            flat.push(p.y);
        }
        flat.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Polyline {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let flat = <Vec<f64>>::deserialize(deserializer)?;
        if flat.len() % 2 != 0 {
            return Err(D::Error::custom("polyline must contain an even number of coordinates"));
        }
        let (pairs, _rest) = flat.as_chunks::<2>();
        let points = pairs.iter().map(|xy| Vec2::new(xy[0], xy[1])).collect();
        Ok(Self(points))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothing_keeps_endpoints_and_pinned_vertices() {
        let polyline = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)]);
        let smoothed = polyline.smoothed(2, &[1]);
        assert_eq!(smoothed.first().copied(), Some(Vec2::new(0.0, 0.0)));
        assert_eq!(smoothed.last().copied(), Some(Vec2::new(10.0, 10.0)));
        assert!(smoothed.contains(&Vec2::new(10.0, 0.0)), "pinned vertex survives");
    }

    #[test]
    fn arc_length_counts_full_and_fractional_segments() {
        let polyline = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)]);
        assert_eq!(polyline.arc_length(1.0), Millimeters::new(10.0));
        assert_eq!(polyline.arc_length(1.5), Millimeters::new(15.0));
        assert_eq!(polyline.arc_length(2.0), Millimeters::new(20.0));
    }

    #[test]
    fn arc_length_is_zero_on_a_degenerate_line() {
        assert_eq!(Polyline::new(vec![Vec2::new(0.0, 0.0)]).arc_length(0.5), Millimeters::ZERO);
    }

    #[test]
    fn smoothing_cuts_an_unpinned_corner() {
        let polyline = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)]);
        let smoothed = polyline.smoothed(1, &[]);
        assert!(!smoothed.contains(&Vec2::new(10.0, 0.0)), "corner is cut");
        assert!(smoothed.len() > polyline.len());
    }

    #[test]
    fn indices_within_finds_only_the_near_points() {
        let polyline = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(20.0, 0.0)]);
        assert_eq!(polyline.indices_within(&[Vec2::new(10.5, 0.4)], Millimeters::new(1.5)), vec![1]);
        assert!(polyline.indices_within(&[Vec2::new(50.0, 50.0)], Millimeters::new(1.5)).is_empty());
    }

    #[test]
    fn smoothed_anchored_pins_points_near_the_anchors() {
        let polyline = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)]);
        let smoothed = polyline.smoothed_anchored(&[Vec2::new(10.0, 0.0)], Millimeters::new(1.5), 2);
        assert!(smoothed.contains(&Vec2::new(10.0, 0.0)), "anchored vertex survives");
    }

    #[test]
    fn axis_param_reports_the_crossing_parameter() {
        let polyline = Polyline::new(vec![Vec2::new(2.0, -1.0), Vec2::new(2.0, 1.0)]);
        let param = polyline.axis_param(Vec2::ZERO, Vec2::new(1.0, 0.0), Millimeters::new(10.0));
        assert_eq!(param, Some(Millimeters::new(2.0)));
    }

    #[test]
    fn axis_param_is_none_beyond_reach() {
        let polyline = Polyline::new(vec![Vec2::new(20.0, -1.0), Vec2::new(20.0, 1.0)]);
        assert_eq!(polyline.axis_param(Vec2::ZERO, Vec2::new(1.0, 0.0), Millimeters::new(10.0)), None);
    }

    #[test]
    fn crossings_detect_intersecting_lines_only() {
        let river = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)]);
        let crossing = Polyline::new(vec![Vec2::new(0.0, 10.0), Vec2::new(10.0, 0.0)]);
        let parallel = Polyline::new(vec![Vec2::new(0.0, 1.0), Vec2::new(10.0, 11.0)]);
        assert!(river.crosses(&crossing));
        assert!(!river.crosses(&parallel));
    }

    #[test]
    fn segments_yield_point_to_point_windows() {
        let polyline = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)]);
        let segments: Vec<Segment<Millimeters>> = polyline.segments().collect();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], Segment::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)));
    }
}
