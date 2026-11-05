use cairo::Context;
use cairo::FontSlant;
use cairo::FontWeight;
use cairo::LineCap;
use qdo36_map::ImportanceClass;
use qdo36_map::Transport;
use std::f64::consts::TAU;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;
use tangible_objects_model::Vec2;

use super::palette::PALETTE;
use crate::ui::style::Alpha;
use crate::ui::style::EdgeSpec;
use crate::ui::style::LabelOrientation;
use crate::ui::style::LabelSpec;
use crate::ui::style::Luminance;
use crate::ui::style::PxPos;
use crate::ui::style::RgbColor;
use crate::ui::style::StationSpec;
use crate::ui::style::StyleRenderer;
use crate::ui::style::WATER_HALF_WIDTH;
use crate::ui::style::WaterBanks;
use crate::ui::style::band_path;
use crate::ui::style::edge_path;
use crate::ui::style::px_polyline_path;
use qdo36_tangibles::ScaleFactor;

/// The default look: flat fills, bold outlines, halftone-tinted
/// water (ARTWORK §2 Comic).
#[derive(Debug, Clone, Copy)]
pub struct ComicStyle;

impl StyleRenderer for ComicStyle {
    fn paper(&self) -> RgbColor {
        PALETTE.paper
    }

    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: &dyn Fn(Vec2<Millimeters>) -> PxPos) {
        // Two bank lines around a darker river body - the Seine
        // reads as a band, not a stroke. The 7 mm band width is
        // map-space, so the river zooms with the map.
        let banks = WaterBanks::new(points, WATER_HALF_WIDTH);
        if banks.left.is_empty() {
            return;
        }
        let left_bank: Vec<PxPos> = banks.left.iter().map(|pos| to_px(*pos)).collect();
        let right_bank: Vec<PxPos> = banks.right.iter().map(|pos| to_px(*pos)).collect();
        band_path(ctx, &left_bank, &right_bank);
        PALETTE.water_fill.apply(ctx);
        ctx.fill().ok();
        ctx.set_line_cap(LineCap::Round);
        ctx.set_line_width(px_per_mm.scale(1.2).max(0.5));
        PALETTE.water.apply(ctx);
        for bank in [&left_bank, &right_bank] {
            px_polyline_path(ctx, bank);
            ctx.stroke().ok();
        }
    }

    fn draw_bridge(&self, ctx: &Context, from: PxPos, to: PxPos, px_per_mm: PixelsPerMm) {
        ctx.set_line_cap(LineCap::Round);
        ctx.move_to(from.x, from.y);
        ctx.line_to(to.x, to.y);
        ctx.set_line_width(px_per_mm.scale(5.0).max(0.5));
        PALETTE.paper.apply(ctx);
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px_per_mm.scale(2.2).max(0.4));
        PALETTE.station.apply(ctx);
        ctx.stroke().ok();
    }

    fn draw_edge(&self, ctx: &Context, edge: &EdgeSpec<'_>) {
        let px = edge.px_per_mm;
        ctx.set_line_cap(LineCap::Round);
        if edge.transport == Transport::Underground {
            ctx.set_dash(&[px.scale(4.0), px.scale(3.0)], 0.0);
        }
        let color = if edge.highlight {
            PALETTE.transport_color(edge.transport, edge.line, edge.map)
        } else {
            PALETTE.edge
        };
        color.apply(ctx);
        ctx.set_line_width(px.scale(2.4).max(0.5));
        edge_path(ctx, edge);
        if edge.wavy {
            // Boat wake: ink rim under the colored wave.
            ctx.set_line_width(px.scale(3.2).max(0.7));
            PALETTE.ink.apply(ctx);
            ctx.stroke_preserve().ok();
            ctx.set_line_width(px.scale(2.2).max(0.5));
            color.apply(ctx);
        }
        if edge.dashed {
            // Metro lane in the combined overlay: dark rim under
            // the dashed color stroke keeps lookalike colors
            // (M1 vs taxi, M6/M12 vs bus) apart.
            ctx.set_dash(&[px.scale(4.0), px.scale(2.5)], 0.0);
            ctx.set_line_width(px.scale(3.4).max(0.7));
            PALETTE.ink.apply(ctx);
            ctx.stroke_preserve().ok();
            ctx.set_line_width(px.scale(2.4).max(0.5));
            color.with_min_luminance(Luminance::new(0.5)).apply(ctx);
        }
        ctx.stroke().ok();
        ctx.set_dash(&[], 0.0);
    }

    fn draw_landmark(&self, ctx: &Context, pos: PxPos, scale: ScaleFactor, px_per_mm: PixelsPerMm) {
        ctx.new_path();
        PALETTE.dim.apply(ctx);
        ctx.set_line_width(1.5);
        ctx.arc(pos.x, pos.y, px_per_mm.scale(2.2 * scale.get()).max(0.5), 0.0, TAU);
        ctx.stroke().ok();
    }

    fn draw_station(&self, ctx: &Context, spec: &StationSpec<'_>) {
        // `show_text` (labels) leaves a current point; `arc` would
        // otherwise insert a stray line_to into the first ring.
        ctx.new_path();
        let pos = spec.pos;
        let px_per_mm = spec.px_per_mm;
        let size = match spec.importance {
            Some(ImportanceClass::Minor) => 0.75,
            Some(ImportanceClass::Hub) => 1.35,
            _ => 1.0,
        };
        let radius = px_per_mm.scale(3.0 * size * spec.station_scale.get()).max(0.4);
        // Fill: inside the metro overlay the disc splits into line
        // color sectors (1 = full disc, 2 = halves, 3 = thirds...).
        if spec.metro_overlay && !spec.lines.is_empty() {
            let sectors = spec.lines.len() as f64;
            for (i, line) in spec.lines.iter().enumerate() {
                let start = -std::f64::consts::FRAC_PI_2 + i as f64 * TAU / sectors;
                ctx.move_to(pos.x, pos.y);
                ctx.arc(pos.x, pos.y, radius, start, start + TAU / sectors);
                ctx.close_path();
                PALETTE.transport_color(Transport::Metro, Some(line), spec.map).apply(ctx);
                ctx.fill().ok();
            }
        } else {
            ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
            PALETTE.station.apply(ctx);
            ctx.fill_preserve().ok();
            ctx.set_line_width(1.4);
            PALETTE.ink.apply(ctx);
            ctx.stroke().ok();
        }
        // One ring per served transport, outermost first.
        for (i, transport) in spec.served.iter().enumerate() {
            ctx.arc(
                pos.x,
                pos.y,
                radius + px_per_mm.scale(1.1 * spec.ring_width.get() * (spec.served.len() - i) as f64).max(0.4),
                0.0,
                TAU,
            );
            PALETTE.base_color(*transport).apply(ctx);
            ctx.set_line_width(0.9 * spec.ring_width.get());
            ctx.stroke().ok();
        }
        if spec.selected {
            ctx.arc(pos.x, pos.y, radius + px_per_mm.scale(3.0).max(0.4), 0.0, TAU);
            PALETTE.select.apply(ctx);
            ctx.set_line_width(2.5);
            ctx.stroke().ok();
        }
    }

    fn draw_label(&self, ctx: &Context, spec: &LabelSpec<'_>) {
        let pos = spec.pos;
        ctx.select_font_face("sans-serif", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(spec.px_per_mm.scale(if spec.inside { 3.4 } else { 4.0 } * spec.label_scale.get()).max(8.0));
        let Ok(extents) = ctx.text_extents(spec.text) else { return };
        let _saved = ctx.save();
        if let LabelOrientation::Radial { center } = spec.orientation {
            let angle = (pos.y - center.y).atan2(pos.x - center.x) - std::f64::consts::FRAC_PI_2;
            ctx.translate(pos.x, pos.y);
            ctx.rotate(angle);
            ctx.translate(-pos.x, -pos.y);
        }
        let y = if spec.inside {
            pos.y + extents.height() / 2.0
        } else {
            pos.y + spec.radius.as_f64() + spec.px_per_mm.scale(5.5).max(10.0)
        };
        ctx.new_path();
        ctx.move_to(pos.x - extents.width() / 2.0, y);
        // In-disc numbers draw in ink - `label` is paper-colored.
        if spec.inside {
            PALETTE.ink.apply(ctx);
        } else {
            PALETTE.label.apply(ctx);
        }
        ctx.show_text(spec.text).ok();
        let _restored = ctx.restore();
    }

    fn draw_anchor(&self, ctx: &Context, pos: PxPos, tracked: bool, px_per_mm: PixelsPerMm) {
        ctx.new_path();
        let radius = px_per_mm.scale(6.0).max(0.5);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        if tracked {
            PALETTE.accent.apply(ctx);
            ctx.fill_preserve().ok();
            RgbColor::new(1.0, 1.0, 1.0).apply(ctx);
        } else {
            PALETTE.dim.apply(ctx);
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
        PALETTE.panel.apply_alpha(ctx, Alpha::new(0.72));
        ctx.rectangle(x, y, box_w, box_h);
        ctx.fill().ok();
        PALETTE.label.apply(ctx);
        ctx.move_to(x + pad_x - extents.x_bearing(), y + pad_y - extents.y_bearing());
        ctx.show_text(text).ok();
    }
}
