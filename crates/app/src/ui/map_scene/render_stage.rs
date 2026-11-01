//! `RenderStage` - the transform/pixel plumbing for one render frame.

use qdo36_tangibles::AnchorPose;
use qdo36_tangibles::MapTransform;
use qdo36_tangibles::SurfaceMm;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::SurfaceFit;
use tangible_objects_model::Vec2;

use crate::ui::map_scene::SceneState;
use crate::ui::style::PxPos;

/// The shared transform/pixel plumbing for one render frame (or one
/// inverse query): `MapTransform` (map-mm -> surface-mm) plus the
/// upstream `SurfaceFit` (surface-mm -> widget-px, incl. display
/// rotation + letterbox).
pub struct RenderStage {
    /// Uniform surface fit: rotation + scale + letterbox offset.
    pub fit: SurfaceFit,
    /// Map-mm -> surface-mm transform.
    transform: MapTransform,
}

impl RenderStage {
    /// Builds the stage for `state` at widget size `width`x`height`
    /// (widget px), anchoring at `pose` (the interpolated pose for
    /// this frame).
    pub fn new(state: &SceneState, pose: AnchorPose, width: Pixels, height: Pixels) -> Self {
        let bounds = state.map.bounds();
        let surface = state.surface.unwrap_or_else(|| SurfaceMm {
            width: Millimeters::new(width.as_f64().max(bounds.width().as_f64())),
            height: Millimeters::new(height.as_f64().max(bounds.height().as_f64())),
        });
        Self {
            fit: SurfaceFit::new(surface.width, surface.height, width, height, state.display_rotation),
            transform: MapTransform::new(pose, &state.config, state.scale),
        }
    }

    /// map-mm -> widget-px.
    pub fn to_px(&self, pos: Vec2<Millimeters>) -> PxPos {
        self.fit.map(self.transform.map_to_surface(pos))
    }

    /// widget-px -> map-mm (exact inverse of `to_px`).
    pub fn to_map(&self, pos: PxPos) -> Vec2<Millimeters> {
        self.transform.surface_to_map(self.fit.unmap(pos))
    }
}
