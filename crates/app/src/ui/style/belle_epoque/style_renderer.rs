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
        // Engraved river: a pale wash band framed by thin sepia
        // bank hairlines, like the rivers on period maps. The 7 mm
        // band width is map-space, so the river zooms with the map.
        let banks = WaterBanks::new(points, WATER_HALF_WIDTH);
        if banks.left.is_empty() {
            return;
        }
        let left_bank: Vec<PxPos> = banks.left.iter().map(|pos| to_px(*pos)).collect();
        let right_bank: Vec<PxPos> = banks.right.iter().map(|pos| to_px(*pos)).collect();
        ctx.set_line_cap(LineCap::Round);
        band_path(ctx, &left_bank, &right_bank);
        PALETTE.water_fill.apply(ctx);
        ctx.fill().ok();
        ctx.set_line_width(px_per_mm.scale(0.6).max(0.3));
        PALETTE.ink.apply_alpha(ctx, Alpha::new(0.55));
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
        edge_path(ctx, edge);
        let color = if edge.highlight {
            PALETTE.transport_color(edge.transport, edge.line, edge.map)
        } else {
            PALETTE.edge
        };
        if edge.transport == Transport::Underground {
            ctx.set_dash(&[px.scale(3.2), px.scale(2.4)], 0.0);
            ctx.set_line_width(px.scale(1.2).max(0.4));
            color.apply_alpha(ctx, Alpha::new(if edge.highlight { 0.6 } else { 0.9 }));
            ctx.stroke().ok();
            ctx.set_dash(&[], 0.0);
            return;
        }
        if edge.dashed {
            // Metro lane in the combined overlay: dashes with the
            // sepia frame keep lookalike colors apart (M1 vs taxi).
            ctx.set_dash(&[px.scale(4.0), px.scale(2.5)], 0.0);
        }
        // Engraving frame: a fine sepia hairline under the route color,
        // so every line reads like a plated double-rule - also the
        // boat wake's rim.
        ctx.set_line_width(px.scale(if edge.wavy { 2.2 } else { 2.6 }).max(0.6));
        PALETTE.ink.apply_alpha(ctx, Alpha::new(if edge.highlight { 0.45 } else { 0.25 }));
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px.scale(1.6).max(0.4));
        color.apply(ctx);
        ctx.stroke().ok();
        ctx.set_dash(&[], 0.0);
    }

    fn draw_landmark(&self, ctx: &Context, pos: PxPos, scale: ScaleFactor, px_per_mm: PixelsPerMm) {
        ctx.new_path();
        let radius = px_per_mm.scale(2.2 * scale.get()).max(0.5);
        PALETTE.ink.apply_alpha(ctx, Alpha::new(0.75));
        ctx.set_line_width(1.0);
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        ctx.stroke().ok();
        PALETTE.accent.apply_alpha(ctx, Alpha::new(0.6));
        ctx.arc(pos.x, pos.y, radius * 0.6, 0.0, TAU);
        ctx.stroke().ok();
    }

    fn draw_station(&self, ctx: &Context, spec: &StationSpec<'_>) {
        // Labels leave a current point; `arc` would otherwise join it.
        ctx.new_path();
        let pos = spec.pos;
        let px_per_mm = spec.px_per_mm;
        let size = match spec.importance {
            Some(ImportanceClass::Minor) => 0.75,
            Some(ImportanceClass::Hub) => 1.35,
            _ => 1.0,
        };
        let radius = px_per_mm.scale(2.6 * size * spec.station_scale.get()).max(0.4);
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
            ctx.set_line_width(1.1);
            PALETTE.ink.apply(ctx);
            ctx.stroke().ok();
        }
        for (i, transport) in spec.served.iter().enumerate() {
            ctx.arc(
                pos.x,
                pos.y,
                radius + px_per_mm.scale(0.8 * spec.ring_width.get() * (spec.served.len() - i) as f64).max(0.4),
                0.0,
                TAU,
            );
            PALETTE.base_color(*transport).apply(ctx);
            ctx.set_line_width(0.7 * spec.ring_width.get());
            ctx.stroke().ok();
        }
        // Ornamental inner ring, like the plated station badges.
        ctx.arc(pos.x, pos.y, radius * 0.68, 0.0, TAU);
        PALETTE.accent.apply_alpha(ctx, Alpha::new(0.8));
        ctx.set_line_width(0.6);
        ctx.stroke().ok();
        if spec.selected {
            ctx.arc(pos.x, pos.y, radius + px_per_mm.scale(2.4).max(0.4), 0.0, TAU);
            PALETTE.select.apply(ctx);
            ctx.set_line_width(1.6);
            ctx.stroke_preserve().ok();
            PALETTE.ink.apply_alpha(ctx, Alpha::new(0.4));
            ctx.set_line_width(0.7);
            ctx.stroke().ok();
        }
    }

    fn draw_label(&self, ctx: &Context, spec: &LabelSpec<'_>) {
        let pos = spec.pos;
        ctx.select_font_face("Cormorant Garamond", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(spec.px_per_mm.scale(if spec.inside { 3.2 } else { 3.6 } * spec.label_scale.get()).max(8.0));
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
            pos.y + spec.radius.as_f64() + spec.px_per_mm.scale(5.2).max(9.5)
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
