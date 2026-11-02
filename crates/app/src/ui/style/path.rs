//! Path helpers shared by the style renderers.

use cairo::Context;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

use crate::ui::style::PxPos;

/// Strokes a polyline path in widget px; points arrive in surface
/// millimeters and `to_px` performs the stage transform.
pub fn polyline_path(ctx: &Context, points: &[Vec2<Millimeters>], to_px: &dyn Fn(Vec2<Millimeters>) -> PxPos) {
    ctx.new_path();
    for (index, point) in points.iter().enumerate() {
        let pos = to_px(*point);
        if index == 0 {
            ctx.move_to(pos.x, pos.y);
        } else {
            ctx.line_to(pos.x, pos.y);
        }
    }
}
