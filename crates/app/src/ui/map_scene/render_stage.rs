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
    /// View pan offset in surface-mm (post-transform).
    pan: Vec2<Millimeters>,
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
            pan: state.pan,
        }
    }

    /// map-mm -> widget-px (transform, then the view pan).
    pub fn to_px(&self, pos: Vec2<Millimeters>) -> PxPos {
        let surface = self.transform.map_to_surface(pos);
        self.fit.map(Vec2::new(surface.x + self.pan.x, surface.y + self.pan.y))
    }

    /// widget-px -> map-mm (exact inverse of `to_px`).
    pub fn to_map(&self, pos: PxPos) -> Vec2<Millimeters> {
        let surface = self.fit.unmap(pos);
        self.transform.surface_to_map(Vec2::new(surface.x - self.pan.x, surface.y - self.pan.y))
    }

    /// The view pan that puts map position `target` on widget
    /// pixel `point` under the current transform (the `goto_*`
    /// tools' centering math).
    pub fn pan_for(&self, target: Vec2<Millimeters>, point: PxPos) -> Vec2<Millimeters> {
        let surface = self.transform.map_to_surface(target);
        let want = self.fit.unmap(point);
        Vec2::new(want.x - surface.x, want.y - surface.y)
    }
}
