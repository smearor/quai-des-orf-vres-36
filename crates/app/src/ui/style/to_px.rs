//! `ToPx` - the map-mm to widget-px projection.

use crate::ui::style::renderer::PxPos;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

/// The scene's mm → widget-px projection: the stage transform
/// (`MapTransform` + px/mm) the styles draw through. `points`
/// projects a whole polygon at once.
#[derive(Clone, Copy)]
pub struct ToPx<'a>(pub &'a dyn Fn(Vec2<Millimeters>) -> PxPos);

impl ToPx<'_> {
    /// One position projected to widget px.
    pub fn pos(&self, mm: Vec2<Millimeters>) -> PxPos {
        (self.0)(mm)
    }

    /// A mm point list's vertices projected to widget px -
    /// `Polygon`/`Polyline` deref into this.
    pub fn points(&self, points: &[Vec2<Millimeters>]) -> Vec<PxPos> {
        points.iter().map(|pos| (self.0)(*pos)).collect()
    }
}
