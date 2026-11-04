//! Path helpers shared by the style renderers.

use cairo::Context;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

use crate::ui::style::EdgeSpec;
use crate::ui::style::PxPos;

/// Builds the edge path: the `route` polyline (or the straight
/// `from` -> `to` chord) shifted per segment by `offsets` - parallel
/// metro lines draw side by side in their own colors.
pub fn edge_path(ctx: &Context, edge: &EdgeSpec<'_>) {
    if edge.route.is_empty() {
        let offset = edge.offsets.first().copied().unwrap_or(0.0);
        let (dx, dy) = (edge.to.x - edge.from.x, edge.to.y - edge.from.y);
        let len = (dx * dx + dy * dy).sqrt().max(1e-6);
        ctx.move_to(edge.from.x - dy / len * offset, edge.from.y + dx / len * offset);
        ctx.line_to(edge.to.x - dy / len * offset, edge.to.y + dx / len * offset);
        return;
    }
    let mut pen_at = f64::NAN;
    for i in 0..edge.route.len().saturating_sub(1) {
        let offset = edge.offsets.get(i).copied().unwrap_or(0.0);
        let (a, b) = (edge.route[i], edge.route[i + 1]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len = (dx * dx + dy * dy).sqrt().max(1e-6);
        let (nx, ny) = (-dy / len * offset, dx / len * offset);
        // Continue the stroke while the offset stays constant;
        // start a new subpath where corridors split or merge.
        if offset != pen_at {
            ctx.move_to(a.x + nx, a.y + ny);
        }
        pen_at = offset;
        ctx.line_to(b.x + nx, b.y + ny);
    }
}

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
