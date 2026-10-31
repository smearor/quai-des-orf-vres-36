use crate::MapError;
use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Axis-aligned bounding box of the map coordinate space.
///
/// The map lives in an abstract millimeter rectangle; the anchor
/// transform in `qdo36-tangibles` places it onto the physical table.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    /// Lower-left corner of the map area.
    pub min: Vec2<Millimeters>,
    /// Upper-right corner of the map area.
    pub max: Vec2<Millimeters>,
}

impl Bounds {
    /// Width of the map area.
    pub fn width(self) -> Millimeters {
        Millimeters::from(self.max.x - self.min.x)
    }

    /// Height of the map area.
    pub fn height(self) -> Millimeters {
        Millimeters::from(self.max.y - self.min.y)
    }

    /// Center of the map area.
    pub fn center(self) -> Vec2<Millimeters> {
        Vec2::new((self.min.x + self.max.x) / 2.0, (self.min.y + self.max.y) / 2.0)
    }

    /// Returns `true` when the point lies inside the bounds (inclusive).
    pub fn contains(self, pos: Vec2<Millimeters>) -> bool {
        (self.min.x..=self.max.x).contains(&pos.x) && (self.min.y..=self.max.y).contains(&pos.y)
    }

    /// Validates that the bounds are finite and non-degenerate.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::InvalidBounds`] for NaN, infinite or
    /// zero/negative-sized bounds.
    pub fn validate(self) -> Result<(), MapError> {
        let finite = [self.min.x, self.min.y, self.max.x, self.max.y].iter().all(|v| v.is_finite());
        if finite && self.width() > Millimeters::default() && self.height() > Millimeters::default() {
            Ok(())
        } else {
            Err(MapError::InvalidBounds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Bounds {
        Bounds {
            min: Vec2::new(10.0, 20.0),
            max: Vec2::new(110.0, 70.0),
        }
    }

    #[test]
    fn geometry() {
        let bounds = bounds();
        assert_eq!(bounds.width().as_f64(), 100.0);
        assert_eq!(bounds.height().as_f64(), 50.0);
        assert_eq!(bounds.center(), Vec2::new(60.0, 45.0));
        assert!(bounds.contains(Vec2::new(60.0, 45.0)));
        assert!(bounds.contains(Vec2::new(10.0, 20.0)));
        assert!(!bounds.contains(Vec2::new(9.9, 45.0)));
    }

    #[test]
    fn rejects_degenerate_bounds() {
        for (min_x, min_y, max_x, max_y) in [(0.0, 0.0, 0.0, 10.0), (0.0, 0.0, 10.0, 0.0), (0.0, 0.0, -5.0, 10.0), (0.0, 0.0, f64::NAN, 1.0)] {
            let bounds = Bounds {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            };
            assert!(matches!(bounds.validate(), Err(MapError::InvalidBounds)), "{bounds:?}");
        }
    }
}
