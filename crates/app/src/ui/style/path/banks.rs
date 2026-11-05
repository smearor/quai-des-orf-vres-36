//! `WaterBanks` - the two bank lines of a water body.

use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// A water body's two bank lines in map mm: `left`/`right` as
/// seen along the course direction.
pub struct WaterBanks {
    /// The bank left of the course direction.
    pub left: Vec<Vec2<Millimeters>>,
    /// The bank right of the course direction.
    pub right: Vec<Vec2<Millimeters>>,
}

impl WaterBanks {
    /// Computes the banks `half_width` off the `points` centerline.
    /// Both banks are empty when the centerline has fewer than two
    /// points. Transform the banks via `to_px` at draw time so the
    /// band zooms and rotates with the map.
    pub fn new(points: &[Vec2<Millimeters>], half_width: Millimeters) -> Self {
        if points.len() < 2 {
            return Self {
                left: Vec::new(),
                right: Vec::new(),
            };
        }
        Self {
            left: offset_polyline(points, half_width.as_f64()),
            right: offset_polyline(points, -half_width.as_f64()),
        }
    }
}

/// Shifts a polyline `distance` units to the left of its direction.
/// Bend vertices average the adjacent segment normals, so the
/// offset stays near-constant without miter spikes.
fn offset_polyline<U: Copy>(points: &[Vec2<U>], distance: f64) -> Vec<Vec2<U>> {
    let normal = |from: Vec2<U>, to: Vec2<U>| {
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        let length = dx.hypot(dy).max(1e-6);
        (-dy / length, dx / length)
    };
    points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let (mut nx, mut ny) = (0.0, 0.0);
            if index > 0 {
                let (x, y) = normal(points[index - 1], *point);
                nx += x;
                ny += y;
            }
            if index + 1 < points.len() {
                let (x, y) = normal(*point, points[index + 1]);
                nx += x;
                ny += y;
            }
            let length = nx.hypot(ny).max(1e-6);
            Vec2::new(point.x + nx / length * distance, point.y + ny / length * distance)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::offset_polyline;
    use crate::ui::style::PxPos;

    #[test]
    fn offsets_straight_line_perpendicular() {
        let line = vec![PxPos::new(0.0, 0.0), PxPos::new(10.0, 0.0)];
        let bank = offset_polyline(&line, 2.0);
        assert_eq!(bank.len(), 2);
        for (point, expected) in bank.iter().zip([(0.0, 2.0), (10.0, 2.0)]) {
            assert!((point.x - expected.0).abs() < 1e-9);
            assert!((point.y - expected.1).abs() < 1e-9);
        }
    }

    #[test]
    fn bend_vertex_averages_segment_normals() {
        let line = vec![PxPos::new(0.0, 0.0), PxPos::new(10.0, 0.0), PxPos::new(10.0, 10.0)];
        let corner = offset_polyline(&line, 2.0)[1];
        // Normals (0,1) + (-1,0) normalize to the (-1,1) diagonal.
        assert!((corner.x - (10.0 - 2.0_f64.sqrt())).abs() < 1e-9);
        assert!((corner.y - 2.0_f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn empty_polyline_offsets_to_empty() {
        assert!(offset_polyline::<tangible_objects_model::Pixels>(&[], 2.0).is_empty());
    }
}
