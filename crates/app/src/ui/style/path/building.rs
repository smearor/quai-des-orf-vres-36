//! Building path helpers shared by the style renderers.

use cairo::Context;
use cairo::FontSlant;
use cairo::FontWeight;
use qdo36_map::Units;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;
use tangible_objects_model::Vec2;

use crate::ui::style::BlockSpec;
use crate::ui::style::CaptionStyle;
use crate::ui::style::DistrictLabel;
use crate::ui::style::HouseMode;
use crate::ui::style::PxPolygon;
use crate::ui::style::PxPos;
use crate::ui::style::StylePalette;
use crate::ui::style::ToPx;

/// How far the roof body shrinks toward the footprint centroid -
/// reads as the hip roof of a house seen from above.
const ROOF_SHRINK: f64 = 0.32;

/// Cairo building paths: `ctx.house_*_path(..)` appends roof
/// outlines and details, `ctx.draw_houses(..)` draws the block's
/// houses, `ctx.district_label(..)`/`ctx.park_label(..)` place the
/// place captions.
pub trait BuildingPathExt {
    /// Appends a house footprint's closed outline to the current
    /// path - for the fill and the outline stroke.
    fn house_outline_path(&self, footprint: &PxPolygon<'_>);
    /// Appends a house's top-view roof details to the current
    /// path: the footprint shrunk toward its centroid reads as
    /// the roof body, quads get the hip diagonals from each outer
    /// corner to its inner corner, and terraced houses (`units`
    /// > 1) get split lines across their front.
    fn house_detail_path(&self, footprint: &PxPolygon<'_>, units: Units);
    /// Appends the outward-facing wall quads of an iso-extruded
    /// footprint: each edge's side extends by `shift` when its
    /// outward normal leans toward the shift direction - the far
    /// side hides under the roof.
    fn iso_wall_path(&self, quad: &PxPolygon<'_>, shift: PxPos);
    /// Centers `name` in italic serif at `pos` (map mm): a wide
    /// `halo` stroke around the glyph outlines lifts the text off
    /// its background, then the glyphs fill in `ink`.
    fn place_label(&self, name: &str, pos: Vec2<Millimeters>, to_px: ToPx<'_>, px_per_mm: PixelsPerMm, style: &CaptionStyle);
    /// Draws the block's houses in `block.house_mode`: flat
    /// top-down footprints, a fixed up-left isometric extrusion,
    /// or a radial one where every roof shifts toward the map
    /// center so the walls face the nearest table edge. The fill
    /// shade varies with width and variant in every mode.
    fn draw_houses(&self, block: &BlockSpec<'_>, palette: &StylePalette, to_px: ToPx<'_>);
    /// Draws a block's district name centered at its deepest
    /// inside point, in italic serif with a `halo` outline so it
    /// stays readable over houses and streets.
    fn district_label(&self, label: &DistrictLabel, to_px: ToPx<'_>, px_per_mm: PixelsPerMm, style: &CaptionStyle);
    /// Draws a park's name centered at its blob, smaller than the
    /// district caption.
    fn park_label(&self, name: &str, pos: Vec2<Millimeters>, to_px: ToPx<'_>, px_per_mm: PixelsPerMm, style: &CaptionStyle);
}

/// The houses' extrusion height in map mm.
const ISO_HEIGHT_MM: Millimeters = Millimeters::new(2.2);

impl BuildingPathExt for Context {
    fn house_outline_path(&self, footprint: &PxPolygon<'_>) {
        if footprint.len() < 3 {
            return;
        }
        self.move_to(footprint[0].x, footprint[0].y);
        for point in &footprint[1..] {
            self.line_to(point.x, point.y);
        }
        self.close_path();
    }

    fn house_detail_path(&self, footprint: &PxPolygon<'_>, units: Units) {
        if footprint.len() < 3 {
            return;
        }
        let inner = footprint.shrunk(ROOF_SHRINK);
        self.move_to(inner[0].x, inner[0].y);
        for point in &inner[1..] {
            self.line_to(point.x, point.y);
        }
        self.close_path();
        if footprint.len() == 4 {
            // Hip diagonals from each outer corner to its inner corner.
            for i in 0..4 {
                self.move_to(footprint[i].x, footprint[i].y);
                self.line_to(inner[i].x, inner[i].y);
            }
            // Unit splits across the front.
            let (front_a, front_b, back_b, back_a) = (footprint[0], footprint[1], footprint[2], footprint[3]);
            for k in 1..units.as_u8() {
                let t = f64::from(k) / units.as_f64();
                self.move_to(front_a.x + (front_b.x - front_a.x) * t, front_a.y + (front_b.y - front_a.y) * t);
                self.line_to(back_a.x + (back_b.x - back_a.x) * t, back_a.y + (back_b.y - back_a.y) * t);
            }
        }
    }

    fn draw_houses(&self, block: &BlockSpec<'_>, palette: &StylePalette, to_px: ToPx<'_>) {
        let height = block.px_per_mm.scale(ISO_HEIGHT_MM);
        let shift = block.extrusion_shift(height, to_px);
        let mut houses: Vec<(&qdo36_map::House, Vec<PxPos>, f64)> = block
            .buildings
            .iter()
            .map(|house| {
                let quad: Vec<PxPos> = to_px.points(&house.footprint);
                let c = PxPolygon(&quad).centroid();
                let (cx, cy) = (c.x, c.y);
                (house, quad, cx * -shift.x + cy * -shift.y)
            })
            .collect();
        // Walls spill opposite the roof shift; the houses nearest the
        // spill edge occlude the walls behind them - far to near.
        houses.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        let isometric = !matches!(block.house_mode, HouseMode::TopDown);
        self.save().ok();
        if isometric && block.outline.len() >= 3 {
            // Walls stop at the block edge, roofs stay inside it.
            let outline: Vec<PxPos> = to_px.points(block.outline);
            self.move_to(outline[0].x, outline[0].y);
            for point in &outline[1..] {
                self.line_to(point.x, point.y);
            }
            self.close_path();
            self.clip();
        }
        for (house, quad, _) in &houses {
            let shade = (house.variant.as_f64() * 0.10 + f64::from(house.units.as_u8().saturating_sub(1)) * 0.05).min(0.45);
            // Each house's own height factor scales the extrusion.
            let v = PxPos::new(shift.x * house.height.as_f64(), shift.y * house.height.as_f64());
            let roof: Vec<PxPos> = quad.iter().map(|p| PxPos::new(p.x + v.x, p.y + v.y)).collect();
            if isometric {
                // Wall quads on the edges facing the spill direction -
                // slightly darker than the roof fill.
                self.iso_wall_path(&PxPolygon(quad), v);
                palette.building.lerp(&palette.ink, 0.30).apply(self);
                self.fill().ok();
            }
            self.house_outline_path(&PxPolygon(&roof));
            palette.building.lerp(&palette.paper, shade).apply(self);
            self.fill_preserve().ok();
            palette.block.apply(self);
            self.set_line_width(block.px_per_mm.scale(Millimeters::new(0.4)).max(Pixels::new(0.2)).as_f64());
            self.stroke().ok();
            self.house_detail_path(&PxPolygon(&roof), house.units);
            palette.roof.apply(self);
            self.set_line_width(block.px_per_mm.scale(Millimeters::new(0.28)).max(Pixels::new(0.15)).as_f64());
            self.stroke().ok();
        }
        self.restore().ok();
    }

    fn district_label(&self, label: &DistrictLabel, to_px: ToPx<'_>, px_per_mm: PixelsPerMm, style: &CaptionStyle) {
        self.place_label(&label.name, label.pos, to_px, px_per_mm, style);
    }

    fn park_label(&self, name: &str, pos: Vec2<Millimeters>, to_px: ToPx<'_>, px_per_mm: PixelsPerMm, style: &CaptionStyle) {
        self.place_label(name, pos, to_px, px_per_mm, style);
    }

    fn iso_wall_path(&self, quad: &PxPolygon<'_>, shift: PxPos) {
        if quad.len() < 3 {
            return;
        }
        let c = quad.centroid();
        let (cx, cy) = (c.x, c.y);
        for (a, b) in quad.segments() {
            let (ex, ey) = (b.x - a.x, b.y - a.y);
            let len = ex.hypot(ey);
            if len < 1e-6 {
                continue;
            }
            // Outward normal: the edge perpendicular pointing away
            // from the footprint centroid.
            let (mut nx, mut ny) = (ey / len, -ex / len);
            if (cx - (a.x + ex / 2.0)) * nx + (cy - (a.y + ey / 2.0)) * ny > 0.0 {
                (nx, ny) = (-nx, -ny);
            }
            // Only walls spilling outward draw; the far side hides
            // under the roof.
            if nx * -shift.x + ny * -shift.y > 0.5 {
                self.move_to(a.x, a.y);
                self.line_to(b.x, b.y);
                self.line_to(b.x + shift.x, b.y + shift.y);
                self.line_to(a.x + shift.x, a.y + shift.y);
                self.close_path();
            }
        }
    }

    fn place_label(&self, name: &str, pos: Vec2<Millimeters>, to_px: ToPx<'_>, px_per_mm: PixelsPerMm, style: &CaptionStyle) {
        self.select_font_face("serif", FontSlant::Italic, FontWeight::Normal);
        let font_px = style.font.as_f64() * px_per_mm.as_f64();
        self.set_font_size(font_px);
        let Ok(extents) = self.text_extents(name) else {
            return;
        };
        let point = to_px.pos(pos);
        // Radial captions face the table edge nearest their position
        // - the same convention the station labels use.
        let angle = style.orientation.angle(point);
        self.save().ok();
        self.translate(point.x, point.y);
        self.rotate(angle.as_f64());
        self.move_to(-extents.width() / 2.0, extents.height() / 3.0);
        self.text_path(name);
        self.set_line_join(cairo::LineJoin::Round);
        self.set_line_width((font_px * 0.28).max(0.8));
        style.halo.apply(self);
        self.stroke_preserve().ok();
        style.ink.apply(self);
        self.fill().ok();
        self.restore().ok();
    }
}
