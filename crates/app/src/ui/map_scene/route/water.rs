//! `WaterRoute` - a route drawn along a water course.

use qdo36_map::Polyline;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// A boat route along a water course: the drawn route polyline
/// and the arc length at which it enters the course - the entry
/// arc anchors the wave phase so edges sharing a water stretch
/// trace matching waves.
pub struct WaterRoute {
    /// The drawn route polyline in map mm.
    pub route: Polyline,
    /// The entry point's arc length along the course in map mm.
    pub entry_arc: Millimeters,
}

impl WaterRoute {
    /// The pier-to-pier course a Batobus edge draws: both station
    /// positions are projected onto the closest water course and
    /// the route follows the waterline between the two projected
    /// points - the boat hugs the river instead of cutting corners
    /// over land. `water` carries the same smoothed courses the
    /// bands draw from. Empty when the map ships no water.
    pub fn along(water: &[Polyline], from: Vec2<Millimeters>, to: Vec2<Millimeters>) -> Self {
        // The closest point on a polyline: (position along the line
        // as segment index + fraction, projected point, approach
        // distance).
        let closest = |points: &Polyline, pos: Vec2<Millimeters>| {
            points
                .nearest_segment(&pos)
                .map(|(index, projection)| (index as f64 + projection.fraction, projection.point, pos.distance(&projection.point)))
        };
        let Some((points, entry, exit)) = water
            .iter()
            .filter_map(|points| {
                let (Some(entry), Some(exit)) = (closest(points, from), closest(points, to)) else {
                    return None;
                };
                Some((points, entry, exit))
            })
            .min_by(|(_, a_entry, a_exit), (_, b_entry, b_exit)| (a_entry.2 + a_exit.2).total_cmp(&(b_entry.2 + b_exit.2)))
        else {
            return Self {
                route: Polyline::default(),
                entry_arc: Millimeters::ZERO,
            };
        };
        // Follow the course from `entry` to `exit` - the inner course
        // vertices run in polyline order when `entry` is upstream, in
        // reverse otherwise.
        let (low, high) = if entry.0 <= exit.0 { (entry, exit) } else { (exit, entry) };
        let i0 = low.0.floor() as usize + 1;
        let i1 = high.0.floor() as usize;
        let inner: &[Vec2<Millimeters>] = if i0 <= i1 { &points[i0..=i1] } else { &[] };
        let mut route = vec![from, entry.1];
        if entry.0 <= exit.0 {
            route.extend_from_slice(inner);
        } else {
            route.extend(inner.iter().rev().copied());
        }
        route.extend_from_slice(&[exit.1, to]);
        // The entry point's arc length along the course anchors the
        // wave phase: edges sharing a stretch draw matching waves.
        let entry_arc = points.arc_length(entry.0);
        Self {
            route: Polyline::new(route),
            entry_arc,
        }
    }
}
