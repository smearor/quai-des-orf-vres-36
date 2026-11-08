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

/// The Neon look: near-black field, glowing double-stroked edges,
/// cyan accents (ARTWORK §2 Neon).
#[derive(Debug, Clone, Copy)]
pub struct NeonStyle;

/// The countryside texture tile: a faint diagonal field grid with
/// dim glow dots - quiet neon field lines over the dark paper.
fn outer_tile() -> Option<ImageSurface> {
    let tile = ImageSurface::create(cairo::Format::ARgb32, 26, 26).ok()?;
    let tile_ctx = Context::new(&tile).ok()?;
    PALETTE.paper.apply(&tile_ctx);
    tile_ctx.paint().ok()?;
    // Diagonal field grid with glow nodes - coarse enough to read
    // at map-fit zoom.
    PALETTE.accent.apply_alpha(&tile_ctx, Alpha::new(0.13));
    tile_ctx.set_line_width(0.8);
    for offset in [0.0, 13.0, 26.0] {
        tile_ctx.move_to(offset, 0.0);
        tile_ctx.line_to(offset - 26.0, 26.0);
    }
    tile_ctx.stroke().ok()?;
    PALETTE.block.apply_alpha(&tile_ctx, Alpha::new(0.6));
    for (x, y) in [(6.5, 6.5), (19.5, 19.5)] {
        tile_ctx.arc(x, y, 1.4, 0.0, TAU);
        tile_ctx.fill().ok()?;
    }
    Some(tile)
}

impl StyleRenderer for NeonStyle {
    fn paper(&self) -> RgbColor {
        PALETTE.paper
    }

    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: ToPx<'_>) {
        // Two glowing bank lines around a dark river body - the
        // Seine reads as a band, not a stroke. The 7 mm band width
        // is map-space, so the river zooms with the map.
        let banks = WaterBanks::new(points, WATER_HALF_WIDTH);
        if banks.left.is_empty() {
            return;
        }
        let left_bank: Vec<PxPos> = to_px.points(&banks.left);
        let right_bank: Vec<PxPos> = to_px.points(&banks.right);
        ctx.set_line_cap(LineCap::Round);
        ctx.band_path(&left_bank, &right_bank);
        PALETTE.water_fill.apply(ctx);
        ctx.fill().ok();
        for bank in [&left_bank, &right_bank] {
            ctx.px_polyline_path(bank);
            ctx.set_line_width(px_per_mm.scale(Millimeters::new(1.8)).max(Pixels::new(0.5)).as_f64());
            PALETTE.water.apply_alpha(ctx, Alpha::new(0.25));
            ctx.stroke_preserve().ok();
            ctx.set_line_width(px_per_mm.scale(Millimeters::new(0.9)).max(Pixels::new(0.4)).as_f64());
            PALETTE.water.apply(ctx);
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
        // Decorative streets crossing the block interior - muted
        // slate, distinct from the underground orange, under the
        // captions.
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
        let color = if edge.highlight {
            PALETTE.transport_color(edge.transport, edge.line, edge.map)
        } else {
            PALETTE.edge
        };
        ctx.set_line_cap(LineCap::Round);
        if edge.wavy {
            // Boat wake: a dark rim under the wave keeps the cyan
            // swell readable over the water band.
            ctx.edge_path(edge);
            ctx.set_line_width(px.scale(Millimeters::new(2.2)).max(Pixels::new(0.5)).as_f64());
            PALETTE.paper.apply(ctx);
            ctx.stroke_preserve().ok();
            ctx.set_line_width(px.scale(Millimeters::new(1.3)).max(Pixels::new(0.4)).as_f64());
            color.apply(ctx);
            ctx.stroke().ok();
            return;
        }
        if edge.dashed {
            // Metro lane in the combined overlay: a dashed
            // line-color stroke with a dark rim separates it from
            // lookalike transport lanes (M1 vs taxi). Dark line
            // colors lift toward white, or M12-vanish-into-paper.
            let color = color.with_min_luminance(Luminance::new(0.5));
            ctx.set_dash(&[px.scale(Millimeters::new(4.0)).as_f64(), px.scale(Millimeters::new(2.5)).as_f64()], 0.0);
            ctx.edge_path(edge);
            ctx.set_line_width(px.scale(Millimeters::new(2.4)).max(Pixels::new(0.5)).as_f64());
            PALETTE.paper.apply(ctx);
            ctx.stroke_preserve().ok();
            ctx.set_line_width(px.scale(Millimeters::new(1.4)).max(Pixels::new(0.4)).as_f64());
            color.apply(ctx);
            ctx.stroke().ok();
            ctx.set_dash(&[], 0.0);
            return;
        }
        ctx.set_line_width(
            px.scale(Millimeters::new(if edge.highlight { 3.6 } else { 1.6 }))
                .max(Pixels::new(0.5))
                .as_f64(),
        );
        color.apply_alpha(ctx, Alpha::new(if edge.highlight { 0.28 } else { 0.9 }));
        ctx.edge_path(edge);
        ctx.stroke_preserve().ok();
        ctx.set_line_width(px.scale(Millimeters::new(1.2)).max(Pixels::new(0.4)).as_f64());
        color.apply(ctx);
        ctx.stroke().ok();
    }

    fn draw_landmark(&self, ctx: &Context, pos: PxPos, scale: ScaleFactor, px_per_mm: PixelsPerMm) {
        ctx.new_path();
        PALETTE.accent.apply_alpha(ctx, Alpha::new(0.5));
        ctx.set_line_width(1.0);
        ctx.arc(pos.x, pos.y, px_per_mm.scale(Millimeters::new(2.0 * scale.get())).max(Pixels::new(0.5)).as_f64(), 0.0, TAU);
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
        let radius = px_per_mm
            .scale(Millimeters::new(2.4 * size * spec.station_scale.get()))
            .max(Pixels::new(0.4))
            .as_f64();
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
            ctx.set_line_width(0.8);
            PALETTE.paper.apply(ctx);
            ctx.stroke().ok();
        }
        for (i, transport) in spec.served.iter().enumerate() {
            ctx.arc(
                pos.x,
                pos.y,
                radius
                    + px_per_mm
                        .scale(Millimeters::new(0.9 * spec.ring_width.get() * (spec.served.len() - i) as f64))
                        .max(Pixels::new(0.4))
                        .as_f64(),
                0.0,
                TAU,
            );
            PALETTE.base_color(*transport).apply(ctx);
            ctx.set_line_width(0.8 * spec.ring_width.get());
            ctx.stroke().ok();
        }
        if spec.selected {
            ctx.arc(pos.x, pos.y, radius + px_per_mm.scale(Millimeters::new(2.6)).max(Pixels::new(0.4)).as_f64(), 0.0, TAU);
            PALETTE.select.apply(ctx);
            ctx.set_line_width(1.6);
            ctx.stroke().ok();
        }
    }

    fn draw_label(&self, ctx: &Context, spec: &LabelSpec<'_>) {
        let pos = spec.pos;
        ctx.select_font_face("monospace", FontSlant::Normal, FontWeight::Normal);
        ctx.set_font_size(
            spec.px_per_mm
                .scale(Millimeters::new(if spec.inside { 3.0 } else { 3.4 } * spec.label_scale.get()))
                .max(Pixels::new(7.0))
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
            pos.y + spec.radius.as_f64() + spec.px_per_mm.scale(Millimeters::new(5.0)).max(Pixels::new(9.0)).as_f64()
        };
        ctx.new_path();
        ctx.move_to(pos.x - extents.width() / 2.0, y);
        // In-disc numbers draw in paper ink - `label` is cyan glow.
        if spec.inside {
            PALETTE.paper.apply(ctx);
        } else {
            PALETTE.label.apply_alpha(ctx, Alpha::new(0.85));
        }
        ctx.show_text(spec.text).ok();
        let _restored = ctx.restore();
    }

    fn draw_anchor(&self, ctx: &Context, pos: PxPos, tracked: bool, px_per_mm: PixelsPerMm) {
        ctx.new_path();
        let radius = px_per_mm.scale(Millimeters::new(5.0)).max(Pixels::new(0.5)).as_f64();
        let color = if tracked { PALETTE.accent } else { PALETTE.dim };
        ctx.arc(pos.x, pos.y, radius, 0.0, TAU);
        color.apply_alpha(ctx, Alpha::new(0.35));
        ctx.set_line_width(4.0);
        ctx.stroke_preserve().ok();
        color.apply(ctx);
        ctx.set_line_width(1.2);
        ctx.stroke().ok();
    }

    fn draw_hint(&self, ctx: &Context, text: &str, width: Pixels, height: Pixels, bottom: Pixels) {
        ctx.select_font_face("monospace", FontSlant::Normal, FontWeight::Bold);
        ctx.set_font_size(16.0);
        let Ok(extents) = ctx.text_extents(text) else { return };
        let (pad_x, pad_y) = (14.0, 10.0);
        let (box_w, box_h) = (extents.width() + 2.0 * pad_x, extents.height() + 2.0 * pad_y);
        let (x, y) = ((width.as_f64() - box_w) / 2.0, height.as_f64() - box_h - 24.0 - bottom.as_f64());
        PALETTE.panel.apply_alpha(ctx, Alpha::new(0.85));
        ctx.rectangle(x, y, box_w, box_h);
        ctx.fill().ok();
        PALETTE.accent.apply(ctx);
        ctx.move_to(x + pad_x - extents.x_bearing(), y + pad_y - extents.y_bearing());
        ctx.show_text(text).ok();
    }
}
