use cairo::Context;
use cairo::FontSlant;
use cairo::FontWeight;
use cairo::LineCap;
use qdo36_map::GameMap;
use qdo36_map::LineName;
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

/// The default look: flat fills, bold outlines, halftone-tinted
/// water (ARTWORK §2 Comic).
#[derive(Debug, Clone, Copy)]
pub struct ComicStyle;

impl StyleRenderer for ComicStyle {
    fn paper(&self) -> RgbColor {
        RgbColor::new(0.10, 0.09, 0.08)
    }

    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: &dyn Fn(Vec2<Millimeters>) -> PxPos) {
        ctx.set_line_width(px_per_mm.scale(6.0));
        ctx.set_line_cap(LineCap::Round);
        RgbColor::new(0.18, 0.32, 0.42).apply(ctx);
        polyline_path(ctx, points, to_px);
        ctx.stroke().ok();
    }

    fn draw_edge(&self, ctx: &Context, edge: &EdgeSpec<'_>) {
        let px = edge.px_per_mm;
        ctx.set_line_cap(LineCap::Round);
        match edge.transport {
            Transport::Underground => {
                ctx.set_dash(&[px.scale(4.0), px.scale(3.0)], 0.0);
                RgbColor::new(0.42, 0.32, 0.22).apply(ctx);
            }
            _ => {
                ctx.set_dash(&[], 0.0);
                transport_color(edge.transport, edge.line, edge.map).apply(ctx);
            }
        }
        ctx.set_line_width(px.scale(2.4).max(0.5));
        ctx.move_to(edge.from.x, edge.from.y);
        ctx.line_to(edge.to.x, edge.to.y);
        ctx.stroke().ok();
        ctx.set_dash(&[], 0.0);
    }

    fn draw_landmark(&self, ctx: &Context, pos: PxPos, px_per_mm: PixelsPerMm) {
        RgbColor::new(0.45, 0.42, 0.38).apply(ctx);
        ctx.set_line_width(1.5);
        ctx.arc(pos.x, pos.y, px_per_mm.scale(2.2).max(0.5), 0.0, TAU);
        ctx.stroke().ok();
    }

    fn draw_station(&self, ctx: &Context, pos: PxPos, selected: bool, px_per_mm: PixelsPerMm) {
        let radius = px_per_mm.scale(3.0).max(0.4);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        RgbColor::new(0.92, 0.88, 0.80).apply(ctx);
        ctx.fill_preserve().ok();
        ctx.set_line_width(1.4);
        RgbColor::new(0.15, 0.13, 0.11).apply(ctx);
        ctx.stroke().ok();
        if selected {
            ctx.arc(pos.x, pos.y, radius + px_per_mm.scale(3.0).max(0.4), 0.0, TAU);
            RgbColor::new(0.95, 0.55, 0.15).apply(ctx);
            ctx.set_line_width(2.5);
            ctx.stroke().ok();
        }
    }

    fn draw_label(&self, ctx: &Context, name: &str, pos: PxPos, radius: Pixels, px_per_mm: PixelsPerMm) {
        ctx.select_font_face("sans-serif", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(px_per_mm.scale(4.0).max(8.0));
        let Ok(extents) = ctx.text_extents(name) else { return };
        ctx.move_to(pos.x - extents.width() / 2.0, pos.y + radius.as_f64() + px_per_mm.scale(5.5).max(10.0));
        RgbColor::new(0.88, 0.85, 0.78).apply(ctx);
        ctx.show_text(name).ok();
    }

    fn draw_anchor(&self, ctx: &Context, pos: PxPos, tracked: bool, px_per_mm: PixelsPerMm) {
        let radius = px_per_mm.scale(6.0).max(0.5);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        if tracked {
            RgbColor::new(0.95, 0.45, 0.20).apply(ctx);
            ctx.fill_preserve().ok();
            RgbColor::new(1.0, 1.0, 1.0).apply(ctx);
        } else {
            RgbColor::new(0.55, 0.50, 0.45).apply(ctx);
        }
        ctx.set_line_width(1.8);
        ctx.stroke().ok();
    }

    fn draw_hint(&self, ctx: &Context, text: &str, width: Pixels, height: Pixels) {
        ctx.select_font_face("sans-serif", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(18.0);
        let Ok(extents) = ctx.text_extents(text) else { return };
        let (pad_x, pad_y) = (14.0, 10.0);
        let (box_w, box_h) = (extents.width() + 2.0 * pad_x, extents.height() + 2.0 * pad_y);
        let (x, y) = ((width.as_f64() - box_w) / 2.0, height.as_f64() - box_h - 24.0);
        RgbColor::new(0.0, 0.0, 0.0).apply_alpha(ctx, Alpha::new(0.72));
        ctx.rectangle(x, y, box_w, box_h);
        ctx.fill().ok();
        RgbColor::new(0.95, 0.85, 0.55).apply(ctx);
        ctx.move_to(x + pad_x - extents.x_bearing(), y + pad_y - extents.y_bearing());
        ctx.show_text(text).ok();
    }
}

/// Edge color per transport; metro edges carry their declared line
/// color, batobus/underground get fixed secret-route colors.
fn transport_color(transport: Transport, line: Option<&LineName>, map: &GameMap) -> RgbColor {
    match transport {
        Transport::Taxi => RgbColor::new(0.96, 0.77, 0.09),
        Transport::Bus => RgbColor::new(0.18, 0.55, 0.34),
        Transport::Batobus => RgbColor::new(0.20, 0.60, 0.86),
        Transport::Underground => RgbColor::new(0.42, 0.32, 0.22),
        Transport::Metro => line
            .and_then(|name| map.meta().lines.get(name))
            .map(|color| {
                let [r, g, b] = color.rgb_u8();
                RgbColor::from_u8(r, g, b)
            })
            .unwrap_or_else(|| RgbColor::new(0.75, 0.22, 0.17)),
    }
}
