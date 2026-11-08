use cairo::Context;
use cairo::FontSlant;
use cairo::FontWeight;
use cairo::ImageSurface;
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
use crate::ui::style::BlockSpec;
use crate::ui::style::BuildingPathExt;
use crate::ui::style::CaptionStyle;
use crate::ui::style::EdgePathExt;
use crate::ui::style::EdgeSpec;
use crate::ui::style::GroundExt;
use crate::ui::style::LabelSpec;
use crate::ui::style::Luminance;
use crate::ui::style::OuterSpec;
use crate::ui::style::PxPos;
use crate::ui::style::RgbColor;
use crate::ui::style::StationSpec;
use crate::ui::style::StyleRenderer;
use crate::ui::style::ToPx;
use crate::ui::style::WATER_HALF_WIDTH;
use crate::ui::style::WaterBanks;
use crate::ui::style::WaterPathExt;
use qdo36_tangibles::ScaleFactor;

/// The default look: flat fills, bold outlines, halftone-tinted
/// water (ARTWORK §2 Comic).
#[derive(Debug, Clone, Copy)]
pub struct ComicStyle;

/// The countryside texture tile: a staggered halftone dot grid on
/// paper - comic print grain, infinitely repeatable.
fn outer_tile() -> Option<ImageSurface> {
    let tile = ImageSurface::create(cairo::Format::ARgb32, 24, 24).ok()?;
    let tile_ctx = Context::new(&tile).ok()?;
    PALETTE.paper.apply(&tile_ctx);
    tile_ctx.paint().ok()?;
    // Offset halftone dots - coarse enough to read at map-fit zoom.
    PALETTE.ink.apply_alpha(&tile_ctx, Alpha::new(0.22));
    for (x, y) in [(6.0, 6.0), (18.0, 18.0)] {
        tile_ctx.arc(x, y, 2.8, 0.0, TAU);
        tile_ctx.fill().ok()?;
    }
    // Secondary dots in a cooler tone - distinct from the ink
    // dots instead of the same color twice.
    PALETTE.water.apply_alpha(&tile_ctx, Alpha::new(0.16));
    for (x, y) in [(6.0, 18.0), (18.0, 6.0)] {
        tile_ctx.arc(x, y, 1.6, 0.0, TAU);
        tile_ctx.fill().ok()?;
    }
    Some(tile)
}

impl StyleRenderer for ComicStyle {
    fn paper(&self) -> RgbColor {
        PALETTE.paper
    }

    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: ToPx<'_>) {
        // Two bank lines around a darker river body - the Seine
        // reads as a band, not a stroke. The 7 mm band width is
        // map-space, so the river zooms with the map.
        let banks = WaterBanks::new(points, WATER_HALF_WIDTH);
        if banks.left.is_empty() {
            return;
        }
        let left_bank: Vec<PxPos> = to_px.points(&banks.left);
        let right_bank: Vec<PxPos> = to_px.points(&banks.right);
        ctx.band_path(&left_bank, &right_bank);
        PALETTE.water_fill.apply(ctx);
        ctx.fill().ok();
        ctx.set_line_cap(LineCap::Round);
        ctx.set_line_width(px_per_mm.scale(Millimeters::new(1.2)).max(Pixels::new(0.5)).as_f64());
        PALETTE.water.apply(ctx);
        for bank in [&left_bank, &right_bank] {
            ctx.px_polyline_path(bank);
            ctx.stroke().ok();
        }
    }

    fn draw_bridge(&self, ctx: &Context, from: PxPos, to: PxPos, px_per_mm: PixelsPerMm) {
        ctx.set_line_cap(LineCap::Round);
        ctx.move_to(from.x, from.y);
        ctx.line_to(to.x, to.y);
        ctx.set_line_width(px_per_mm.scale(Millimeters::new(5.0)).max(Pixels::new(0.5)).as_f64());
        PALETTE.paper.apply(ctx);
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px_per_mm.scale(Millimeters::new(2.2)).max(Pixels::new(0.4)).as_f64());
        PALETTE.station.apply(ctx);
        ctx.stroke().ok();
    }

    fn draw_outer(&self, ctx: &Context, spec: &OuterSpec<'_>, to_px: ToPx<'_>) {
        if let Some(tile) = outer_tile() {
            let pin = to_px.pos(spec.pin);
            ctx.tile_fill(&tile, &pin, spec.width.as_f64(), spec.height.as_f64());
        }
        ctx.fill_ground(spec.ground, &PALETTE.paper, to_px);
    }

    fn draw_block(&self, ctx: &Context, block: &BlockSpec<'_>, to_px: ToPx<'_>) {
        ctx.set_line_join(cairo::LineJoin::Round);
        // Parks first: green blobs under the houses (which avoid
        // them anyway); the names draw later, over the streets.
        for park in block.parks {
            let blob: Vec<PxPos> = to_px.points(&park.polygon);
            ctx.px_polyline_path(&blob);
            ctx.close_path();
            PALETTE.park.apply(ctx);
            ctx.fill_preserve().ok();
            PALETTE.block.apply_alpha(ctx, Alpha::new(0.4));
            ctx.set_line_width(block.px_per_mm.scale(Millimeters::new(0.4)).max(Pixels::new(0.2)).as_f64());
            ctx.stroke().ok();
        }
        // Houses: raised fill over the paper, roof details in the
        // `roof` tone (contrast against the fill), block-ink
        // outline. Fill shade varies with width and variant.
        ctx.draw_houses(block, &PALETTE, to_px);
        // Decorative streets crossing the block interior - muted,
        // under the captions.
        ctx.set_line_width(block.px_per_mm.scale(Millimeters::new(1.0)).max(Pixels::new(0.3)).as_f64());
        for street in block.streets {
            let path: Vec<PxPos> = to_px.points(street);
            ctx.px_polyline_path(&path);
            PALETTE.street.apply_alpha(ctx, Alpha::new(0.7));
            ctx.stroke().ok();
        }
        // Park names over the streets, then the district caption.
        for park in block.parks {
            ctx.park_label(
                park.name.as_str(),
                park.polygon.centroid(),
                to_px,
                block.px_per_mm,
                &CaptionStyle::park(PALETTE.paper, PALETTE.label, block.park_orientation),
            );
        }
        if let Some(label) = &block.label {
            ctx.district_label(
                label,
                to_px,
                block.px_per_mm,
                &CaptionStyle::district(PALETTE.paper, PALETTE.label, block.district_orientation),
            );
        }
        // Block outline on top.
        if block.outline.len() >= 3 {
            let outline: Vec<PxPos> = to_px.points(block.outline);
            ctx.px_polyline_path(&outline);
            ctx.close_path();
            ctx.set_line_width(block.px_per_mm.scale(Millimeters::new(0.6)).max(Pixels::new(0.3)).as_f64());
            PALETTE.block.apply(ctx);
            ctx.stroke().ok();
        }
    }

    fn draw_edge(&self, ctx: &Context, edge: &EdgeSpec<'_>) {
        let px = edge.px_per_mm;
        ctx.set_line_cap(LineCap::Round);
        if edge.transport == Transport::Underground {
            ctx.set_dash(&[px.scale(Millimeters::new(4.0)).as_f64(), px.scale(Millimeters::new(3.0)).as_f64()], 0.0);
        }
        let color = if edge.highlight {
            PALETTE.transport_color(edge.transport, edge.line, edge.map)
        } else {
            PALETTE.edge
        };
        color.apply(ctx);
        ctx.set_line_width(px.scale(Millimeters::new(2.4)).max(Pixels::new(0.5)).as_f64());
        ctx.edge_path(edge);
        if edge.wavy {
            // Boat wake: ink rim under the colored wave.
            ctx.set_line_width(px.scale(Millimeters::new(3.2)).max(Pixels::new(0.7)).as_f64());
            PALETTE.ink.apply(ctx);
            ctx.stroke_preserve().ok();
            ctx.set_line_width(px.scale(Millimeters::new(2.2)).max(Pixels::new(0.5)).as_f64());
            color.apply(ctx);
        }
        if edge.dashed {
            // Metro lane in the combined overlay: dark rim under
            // the dashed color stroke keeps lookalike colors
            // (M1 vs taxi, M6/M12 vs bus) apart.
            ctx.set_dash(&[px.scale(Millimeters::new(4.0)).as_f64(), px.scale(Millimeters::new(2.5)).as_f64()], 0.0);
            ctx.set_line_width(px.scale(Millimeters::new(3.4)).max(Pixels::new(0.7)).as_f64());
            PALETTE.ink.apply(ctx);
            ctx.stroke_preserve().ok();
            ctx.set_line_width(px.scale(Millimeters::new(2.4)).max(Pixels::new(0.5)).as_f64());
            color.with_min_luminance(Luminance::new(0.5)).apply(ctx);
        }
        ctx.stroke().ok();
        ctx.set_dash(&[], 0.0);
    }

    fn draw_landmark(&self, ctx: &Context, pos: PxPos, scale: ScaleFactor, px_per_mm: PixelsPerMm) {
        ctx.new_path();
        PALETTE.dim.apply(ctx);
        ctx.set_line_width(1.5);
        ctx.arc(pos.x, pos.y, px_per_mm.scale(Millimeters::new(2.2 * scale.get())).max(Pixels::new(0.5)).as_f64(), 0.0, TAU);
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
        let radius = px_per_mm
            .scale(Millimeters::new(3.0 * size * spec.station_scale.get()))
            .max(Pixels::new(0.4))
            .as_f64();
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
                radius
                    + px_per_mm
                        .scale(Millimeters::new(1.1 * spec.ring_width.get() * (spec.served.len() - i) as f64))
                        .max(Pixels::new(0.4))
                        .as_f64(),
                0.0,
                TAU,
            );
            PALETTE.base_color(*transport).apply(ctx);
            ctx.set_line_width(0.9 * spec.ring_width.get());
            ctx.stroke().ok();
        }
        if spec.selected {
            ctx.arc(pos.x, pos.y, radius + px_per_mm.scale(Millimeters::new(3.0)).max(Pixels::new(0.4)).as_f64(), 0.0, TAU);
            PALETTE.select.apply(ctx);
            ctx.set_line_width(2.5);
            ctx.stroke().ok();
        }
    }

    fn draw_label(&self, ctx: &Context, spec: &LabelSpec<'_>) {
        let pos = spec.pos;
        ctx.select_font_face("sans-serif", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(
            spec.px_per_mm
                .scale(Millimeters::new(if spec.inside { 3.4 } else { 4.0 } * spec.label_scale.get()))
                .max(Pixels::new(8.0))
                .as_f64(),
        );
        let Ok(extents) = ctx.text_extents(spec.text) else { return };
        let _saved = ctx.save();
        let angle = spec.orientation.angle(pos);
        if !angle.is_zero() {
            ctx.translate(pos.x, pos.y);
            ctx.rotate(angle.as_f64());
            ctx.translate(-pos.x, -pos.y);
        }
        let y = if spec.inside {
            pos.y + extents.height() / 2.0
        } else {
            pos.y + spec.radius.as_f64() + spec.px_per_mm.scale(Millimeters::new(5.5)).max(Pixels::new(10.0)).as_f64()
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
        let radius = px_per_mm.scale(Millimeters::new(6.0)).max(Pixels::new(0.5)).as_f64();
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

    fn draw_hint(&self, ctx: &Context, text: &str, width: Pixels, height: Pixels, bottom: Pixels) {
        ctx.select_font_face("sans-serif", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(18.0);
        let Ok(extents) = ctx.text_extents(text) else { return };
        let (pad_x, pad_y) = (14.0, 10.0);
        let (box_w, box_h) = (extents.width() + 2.0 * pad_x, extents.height() + 2.0 * pad_y);
        let (x, y) = ((width.as_f64() - box_w) / 2.0, height.as_f64() - box_h - 24.0 - bottom.as_f64());
        PALETTE.panel.apply_alpha(ctx, Alpha::new(0.72));
        ctx.rectangle(x, y, box_w, box_h);
        ctx.fill().ok();
        PALETTE.label.apply(ctx);
        ctx.move_to(x + pad_x - extents.x_bearing(), y + pad_y - extents.y_bearing());
        ctx.show_text(text).ok();
    }
}
