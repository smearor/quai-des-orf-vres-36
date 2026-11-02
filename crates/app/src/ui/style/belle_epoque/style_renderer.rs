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

use super::palette::PALETTE;
use crate::ui::style::Alpha;
use crate::ui::style::EdgeSpec;
use crate::ui::style::PxPos;
use crate::ui::style::RgbColor;
use crate::ui::style::StyleRenderer;
use crate::ui::style::polyline_path;

/// The Belle-Époque look: parchment field, engraving-style double
/// strokes in sepia ink, gold accents and serif labels (ARTWORK §2
/// Belle Époque - the same voice as the generated masters).
#[derive(Debug, Clone, Copy)]
pub struct BelleEpoqueStyle;

impl StyleRenderer for BelleEpoqueStyle {
    fn paper(&self) -> RgbColor {
        PALETTE.paper
    }

    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: &dyn Fn(Vec2<Millimeters>) -> PxPos) {
        ctx.set_line_cap(LineCap::Round);
        // Engraved river: a pale wash band with a thin sepia hairline
        // down the middle, like the bank lines on period maps.
        polyline_path(ctx, points, to_px);
        ctx.set_line_width(px_per_mm.scale(6.0));
        PALETTE.water.apply(ctx);
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px_per_mm.scale(0.6).max(0.3));
        PALETTE.ink.apply_alpha(ctx, Alpha::new(0.55));
        ctx.stroke().ok();
    }

    fn draw_edge(&self, ctx: &Context, edge: &EdgeSpec<'_>) {
        let px = edge.px_per_mm;
        ctx.set_line_cap(LineCap::Round);
        ctx.move_to(edge.from.x, edge.from.y);
        ctx.line_to(edge.to.x, edge.to.y);
        if edge.transport == Transport::Underground {
            ctx.set_dash(&[px.scale(3.2), px.scale(2.4)], 0.0);
            ctx.set_line_width(px.scale(1.2).max(0.4));
            PALETTE.underground.apply_alpha(ctx, Alpha::new(0.6));
            ctx.stroke().ok();
            ctx.set_dash(&[], 0.0);
            return;
        }
        // Engraving frame: a fine sepia hairline under the route color,
        // so every line reads like a plated double-rule.
        ctx.set_line_width(px.scale(2.6).max(0.6));
        PALETTE.ink.apply_alpha(ctx, Alpha::new(0.45));
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px.scale(1.6).max(0.4));
        PALETTE.transport_color(edge.transport, edge.line, edge.map).apply(ctx);
        ctx.stroke().ok();
    }

    fn draw_landmark(&self, ctx: &Context, pos: PxPos, px_per_mm: PixelsPerMm) {
        let radius = px_per_mm.scale(2.2).max(0.5);
        PALETTE.ink.apply_alpha(ctx, Alpha::new(0.75));
        ctx.set_line_width(1.0);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        ctx.stroke().ok();
        PALETTE.accent.apply_alpha(ctx, Alpha::new(0.6));
        ctx.arc(pos.x, pos.y, radius * 0.6, 0.0, TAU);
        ctx.stroke().ok();
    }

    fn draw_station(&self, ctx: &Context, pos: PxPos, selected: bool, px_per_mm: PixelsPerMm) {
        let radius = px_per_mm.scale(2.6).max(0.4);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        PALETTE.station.apply(ctx);
        ctx.fill_preserve().ok();
        ctx.set_line_width(1.1);
        PALETTE.ink.apply(ctx);
        ctx.stroke().ok();
        // Ornamental inner ring, like the plated station badges.
        ctx.arc(pos.x, pos.y, radius * 0.68, 0.0, TAU);
        PALETTE.accent.apply_alpha(ctx, Alpha::new(0.8));
        ctx.set_line_width(0.6);
        ctx.stroke().ok();
        if selected {
            ctx.arc(pos.x, pos.y, radius + px_per_mm.scale(2.4).max(0.4), 0.0, TAU);
            PALETTE.select.apply(ctx);
            ctx.set_line_width(1.6);
            ctx.stroke_preserve().ok();
            PALETTE.ink.apply_alpha(ctx, Alpha::new(0.4));
            ctx.set_line_width(0.7);
            ctx.stroke().ok();
        }
    }

    fn draw_label(&self, ctx: &Context, name: &str, pos: PxPos, radius: Pixels, px_per_mm: PixelsPerMm) {
        ctx.select_font_face("Cormorant Garamond", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(px_per_mm.scale(3.6).max(8.0));
        let Ok(extents) = ctx.text_extents(name) else { return };
        ctx.move_to(pos.x - extents.width() / 2.0, pos.y + radius.as_f64() + px_per_mm.scale(5.2).max(9.5));
        PALETTE.label.apply(ctx);
        ctx.show_text(name).ok();
    }

    fn draw_anchor(&self, ctx: &Context, pos: PxPos, tracked: bool, px_per_mm: PixelsPerMm) {
        let radius = px_per_mm.scale(5.5).max(0.5);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        if tracked {
            PALETTE.ink.apply(ctx);
            ctx.fill_preserve().ok();
            PALETTE.accent.apply(ctx);
        } else {
            PALETTE.dim.apply_alpha(ctx, Alpha::new(0.7));
        }
        ctx.set_line_width(1.6);
        ctx.stroke().ok();
    }

    fn draw_hint(&self, ctx: &Context, text: &str, width: Pixels, height: Pixels) {
        ctx.select_font_face("Cormorant Garamond", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(17.0);
        let Ok(extents) = ctx.text_extents(text) else { return };
        let (pad_x, pad_y) = (16.0, 11.0);
        let (box_w, box_h) = (extents.width() + 2.0 * pad_x, extents.height() + 2.0 * pad_y);
        let (x, y) = ((width.as_f64() - box_w) / 2.0, height.as_f64() - box_h - 24.0);
        PALETTE.panel.apply_alpha(ctx, Alpha::new(0.94));
        ctx.rectangle(x, y, box_w, box_h);
        ctx.fill().ok();
        PALETTE.accent.apply(ctx);
        ctx.rectangle(x, y, box_w, box_h);
        ctx.set_line_width(1.5);
        ctx.stroke().ok();
        ctx.rectangle(x + 2.5, y + 2.5, box_w - 5.0, box_h - 5.0);
        ctx.set_line_width(0.7);
        ctx.stroke().ok();
        PALETTE.label.apply(ctx);
        ctx.move_to(x + pad_x - extents.x_bearing(), y + pad_y - extents.y_bearing());
        ctx.show_text(text).ok();
    }
}
