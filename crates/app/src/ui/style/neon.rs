use cairo::Context;
use cairo::FontSlant;
use cairo::FontWeight;
use cairo::LineCap;
use qdo36_map::Transport;
use std::f64::consts::TAU;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;
use tangible_objects_model::Vec2;

use crate::ui::style::Alpha;
use crate::ui::style::EdgeSpec;
use crate::ui::style::PxPos;
use crate::ui::style::RgbColor;
use crate::ui::style::StyleRenderer;
use crate::ui::style::polyline_path;

/// The Neon look (rough): near-black field, glowing double-stroked
/// edges, cyan accents - proves the style seam (ARTWORK §2 Neon).
#[derive(Debug, Clone, Copy)]
pub struct NeonStyle;

/// Accent colors.
const GLOW: RgbColor = RgbColor::new(0.10, 0.95, 1.0);
const DIM: RgbColor = RgbColor::new(0.30, 0.45, 0.50);
const SELECT: RgbColor = RgbColor::new(1.0, 0.30, 0.85);

impl StyleRenderer for NeonStyle {
    fn paper(&self) -> RgbColor {
        RgbColor::new(0.02, 0.03, 0.05)
    }

    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: &dyn Fn(Vec2<Millimeters>) -> PxPos) {
        ctx.set_line_cap(LineCap::Round);
        polyline_path(ctx, points, to_px);
        ctx.set_line_width(px_per_mm.scale(6.0));
        GLOW.apply_alpha(ctx, Alpha::new(0.25));
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px_per_mm.scale(1.6).max(0.4));
        GLOW.apply(ctx);
        ctx.stroke().ok();
    }

    fn draw_edge(&self, ctx: &Context, edge: &EdgeSpec<'_>) {
        let color = transport_color(edge.transport);
        let px = edge.px_per_mm;
        ctx.set_line_cap(LineCap::Round);
        // Glow pass: wide translucent stroke, then the core line.
        ctx.set_line_width(px.scale(3.6).max(0.5));
        color.apply_alpha(ctx, Alpha::new(0.28));
        ctx.move_to(edge.from.x, edge.from.y);
        ctx.line_to(edge.to.x, edge.to.y);
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px.scale(1.2).max(0.4));
        color.apply(ctx);
        ctx.stroke().ok();
    }

    fn draw_landmark(&self, ctx: &Context, pos: PxPos, px_per_mm: PixelsPerMm) {
        GLOW.apply_alpha(ctx, Alpha::new(0.5));
        ctx.set_line_width(1.0);
        ctx.arc(pos.x, pos.y, px_per_mm.scale(2.0).max(0.5), 0.0, TAU);
        ctx.stroke().ok();
    }

    fn draw_station(&self, ctx: &Context, pos: PxPos, selected: bool, px_per_mm: PixelsPerMm) {
        let radius = px_per_mm.scale(2.4).max(0.4);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        GLOW.apply(ctx);
        ctx.fill_preserve().ok();
        ctx.set_line_width(0.8);
        self.paper().apply(ctx);
        ctx.stroke().ok();
        if selected {
            ctx.arc(pos.x, pos.y, radius + px_per_mm.scale(2.6).max(0.4), 0.0, TAU);
            SELECT.apply(ctx);
            ctx.set_line_width(1.6);
            ctx.stroke().ok();
        }
    }

    fn draw_label(&self, ctx: &Context, name: &str, pos: PxPos, radius: Pixels, px_per_mm: PixelsPerMm) {
        ctx.select_font_face("monospace", FontSlant::Normal, FontWeight::Normal);
        ctx.set_font_size(px_per_mm.scale(3.4).max(7.0));
        let Ok(extents) = ctx.text_extents(name) else { return };
        ctx.move_to(pos.x - extents.width() / 2.0, pos.y + radius.as_f64() + px_per_mm.scale(5.0).max(9.0));
        RgbColor::new(0.40, 0.85, 0.95).apply_alpha(ctx, Alpha::new(0.85));
        ctx.show_text(name).ok();
    }

    fn draw_anchor(&self, ctx: &Context, pos: PxPos, tracked: bool, px_per_mm: PixelsPerMm) {
        let radius = px_per_mm.scale(5.0).max(0.5);
        let color = if tracked { GLOW } else { DIM };
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        color.apply_alpha(ctx, Alpha::new(0.35));
        ctx.set_line_width(4.0);
        ctx.stroke_preserve().ok();
        color.apply(ctx);
        ctx.set_line_width(1.2);
        ctx.stroke().ok();
    }

    fn draw_hint(&self, ctx: &Context, text: &str, width: Pixels, height: Pixels) {
        ctx.select_font_face("monospace", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(16.0);
        let Ok(extents) = ctx.text_extents(text) else { return };
        let (pad_x, pad_y) = (14.0, 10.0);
        let (box_w, box_h) = (extents.width() + 2.0 * pad_x, extents.height() + 2.0 * pad_y);
        let (x, y) = ((width.as_f64() - box_w) / 2.0, height.as_f64() - box_h - 24.0);
        RgbColor::new(0.0, 0.02, 0.04).apply_alpha(ctx, Alpha::new(0.85));
        ctx.rectangle(x, y, box_w, box_h);
        ctx.fill().ok();
        GLOW.apply(ctx);
        ctx.move_to(x + pad_x - extents.x_bearing(), y + pad_y - extents.y_bearing());
        ctx.show_text(text).ok();
    }
}

/// Neon edge palette per transport (uniform glow family).
fn transport_color(transport: Transport) -> RgbColor {
    match transport {
        Transport::Taxi => RgbColor::new(1.0, 0.85, 0.10),
        Transport::Bus => RgbColor::new(0.20, 0.95, 0.40),
        Transport::Metro => RgbColor::new(0.15, 0.65, 1.0),
        Transport::Batobus => RgbColor::new(0.0, 0.85, 0.95),
        Transport::Underground => RgbColor::new(1.0, 0.45, 0.15),
    }
}
