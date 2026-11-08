//! `WaterBanks` - the two bank lines of a water body.

use crate::map::Polyline;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Half the channel a water course claims on each side (mm) -
/// the setback water `BlockCut`s keep from the course centerline.
pub const WATER_HALF_WIDTH: Millimeters = Millimeters::new(3.5);

/// A water body's two bank lines in map mm: `left`/`right` as
/// seen along the course direction.
pub struct WaterBanks {
    /// The bank left of the course direction.
    pub left: Polyline,
    /// The bank right of the course direction.
    pub right: Polyline,
}

impl WaterBanks {
    /// Computes the banks `half_width` off the `points` centerline.
    /// Both banks are empty when the centerline has fewer than two
    /// points.
    pub fn new(points: &[Vec2<Millimeters>], half_width: Millimeters) -> Self {
        if points.len() < 2 {
            return Self {
                left: Polyline::default(),
                right: Polyline::default(),
            };
        }
        let course = Polyline::new(points.to_vec());
        Self {
            left: course.offset(half_width),
            right: course.offset(-half_width),
        }
    }

    /// The widest absolute parameter `t` where the line `center +
    /// axis * t` meets either bank within `reach` - `Millimeters::ZERO`
    /// when neither bank is reached.
    pub fn crossing_span(&self, center: Vec2<Millimeters>, axis: Vec2<Millimeters>, reach: Millimeters) -> Millimeters {
        [&self.left, &self.right]
            .into_iter()
            .filter_map(|bank| bank.axis_param(center, axis, reach))
            .map(Millimeters::abs)
            .fold(Millimeters::ZERO, Millimeters::max)
    }
}

#[cfg(test)]
mod tests {
    use crate::map::Polyline;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::Vec2;

    #[test]
    fn offsets_straight_line_perpendicular() {
        let line = vec![Vec2::<Millimeters>::new(0.0, 0.0), Vec2::new(10.0, 0.0)];
        let bank = Polyline::new(line).offset(Millimeters::new(2.0));
        assert_eq!(bank.len(), 2);
        for (point, expected) in bank.iter().zip([(0.0, 2.0), (10.0, 2.0)]) {
            assert!((point.x - expected.0).abs() < 1e-9);
            assert!((point.y - expected.1).abs() < 1e-9);
        }
    }

    #[test]
    fn bend_vertex_averages_segment_normals() {
        let line = vec![Vec2::<Millimeters>::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        let corner = Polyline::new(line).offset(Millimeters::new(2.0))[1];
        // Normals (0,1) + (-1,0) normalize to the (-1,1) diagonal.
        assert!((corner.x - (10.0 - 2.0_f64.sqrt())).abs() < 1e-9);
        assert!((corner.y - 2.0_f64.sqrt()).abs() < 1e-9);
    }
}
