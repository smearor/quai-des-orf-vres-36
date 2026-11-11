use crate::AnchorConfig;
use crate::AnchorPose;
use crate::MapScale;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::Vec2;

/// Affine transform between map millimeters and surface millimeters,
/// driven by the anchor pose.
///
/// `surface = anchor_center + R(anchor_rot + heading_offset) * S * (map - anchor_map)`
///
/// Moving the tower pans the map; rotating it rotates the map; the
/// inverse maps surface contacts back into map space for hit-testing
/// and pawn snap. Distinct from a fixed-pivot widget rotation
/// (`smearor-wrot-rotation` is not suitable here).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapTransform {
    /// Zoom: surface-mm per map-mm (overlay zoom controls change it).
    pub scale: MapScale,
    /// Total applied rotation (anchor pose + heading offset).
    pub rotation_rad: RotationRad,
    /// Anchor point in map space.
    anchor_map: Vec2<Millimeters>,
    /// Anchor center in surface space.
    anchor_surface: Vec2<Millimeters>,
}

impl MapTransform {
    /// Builds the transform from the measured anchor pose and the
    /// calibration config.
    pub fn new(pose: AnchorPose, config: &AnchorConfig, scale: MapScale) -> Self {
        Self {
            scale,
            rotation_rad: pose.rotation_rad + config.heading_offset,
            anchor_map: config.anchor_map_pos,
            anchor_surface: pose.center,
        }
    }

    /// Map millimeters -> surface millimeters.
    pub fn map_to_surface(&self, pos: Vec2<Millimeters>) -> Vec2<Millimeters> {
        self.anchor_surface + ((pos - self.anchor_map) * self.scale.factor()).rotated(self.rotation_rad)
    }

    /// Surface millimeters -> map millimeters (inverse of
    /// [`Self::map_to_surface`]).
    ///
    /// Used for touch hit-testing and pawn snap queries.
    pub fn surface_to_map(&self, pos: Vec2<Millimeters>) -> Vec2<Millimeters> {
        self.anchor_map + (pos - self.anchor_surface).rotated(-self.rotation_rad) / self.scale.factor()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tangible_objects_model::Region;

    fn config() -> AnchorConfig {
        AnchorConfig {
            anchor_map_pos: Vec2::new(100.0, 100.0),
            heading_offset: RotationRad::ZERO,
            base_scale: MapScale::default(),
            map_bounds: Region {
                x_min: Millimeters::ZERO,
                y_min: Millimeters::ZERO,
                x_max: Millimeters::new(600.0),
                y_max: Millimeters::new(420.0),
            },
        }
    }

    #[test]
    fn roundtrip_identity() {
        let transform = MapTransform::new(
            AnchorPose {
                center: Vec2::new(300.0, 200.0),
                rotation_rad: RotationRad::new(0.7),
            },
            &config(),
            MapScale::try_new(1.25).unwrap_or_default(),
        );
        let pos = Vec2::new(42.0, 17.0);
        let back = transform.surface_to_map(transform.map_to_surface(pos));
        assert!((back.x - pos.x).abs() < 1e-9 && (back.y - pos.y).abs() < 1e-9, "{back:?} != {pos:?}");
    }

    #[test]
    fn anchor_maps_to_itself() {
        let transform = MapTransform::new(
            AnchorPose {
                center: Vec2::new(300.0, 200.0),
                rotation_rad: RotationRad::ZERO,
            },
            &config(),
            MapScale::ONE,
        );
        let surface = transform.map_to_surface(transform.anchor_map);
        assert_eq!(surface, transform.anchor_surface);
    }

    #[test]
    fn pure_rotation() {
        let transform = MapTransform::new(
            AnchorPose {
                center: Vec2::ZERO,
                rotation_rad: RotationRad::new(std::f64::consts::FRAC_PI_2),
            },
            &config(),
            MapScale::ONE,
        );
        let surface = transform.map_to_surface(Vec2::new(200.0, 100.0)); // 100 right of anchor, 0 below
        assert!((surface.x - 0.0).abs() < 1e-9 && (surface.y - 100.0).abs() < 1e-9, "90° rotation maps +x to +y: {surface:?}");
    }
}
