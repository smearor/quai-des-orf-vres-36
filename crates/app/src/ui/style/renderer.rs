//! `StyleRenderer` - the draw interface a map style implements.

use crate::ui::style::BlockSpec;
use crate::ui::style::EdgeSpec;
use crate::ui::style::LabelSpec;
use crate::ui::style::OuterSpec;
use crate::ui::style::RgbColor;
use crate::ui::style::StationSpec;
use crate::ui::style::ToPx;
use cairo::Context;
use qdo36_tangibles::ScaleFactor;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;
use tangible_objects_model::Vec2;

/// A position in widget pixels - `to_px` produces these, the style
/// draws them.
pub type PxPos = Vec2<Pixels>;

/// How the scene's elements are drawn.
///
/// All coordinates arrive in widget px - the stage (`MapTransform` +
/// px/mm) already applied. `px_per_mm` scales stroke widths.
pub trait StyleRenderer {
    /// The paper color the scene is painted on.
    fn paper(&self) -> RgbColor;

    /// The countryside outside the city: the style's infinitely
    /// repeated texture everywhere the ground polygons don't
    /// cover. Drawn right after the paper, under water, blocks,
    /// lines and landmarks.
    fn draw_outer(&self, ctx: &Context, outer: &OuterSpec<'_>, to_px: ToPx<'_>);

    /// A water polyline (river, canal).
    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: ToPx<'_>);

    /// A bridge deck - the short plank a road edge crosses the
    /// water on, drawn under the edge line.
    fn draw_bridge(&self, ctx: &Context, from: PxPos, to: PxPos, px_per_mm: PixelsPerMm);

    /// A city block - the outline of a free area enclosed by
    /// streets, river and border plus its procedural buildings.
    fn draw_block(&self, ctx: &Context, block: &BlockSpec<'_>, to_px: ToPx<'_>);

    /// A station pair's edge.
    fn draw_edge(&self, ctx: &Context, edge: &EdgeSpec<'_>);

    /// A backdrop landmark marker.
    fn draw_landmark(&self, ctx: &Context, pos: PxPos, scale: ScaleFactor, px_per_mm: PixelsPerMm);

    /// A station node (fill + outline + one ring per served
    /// transport + line pie fill in the metro overlay +
    /// selection ring).
    fn draw_station(&self, ctx: &Context, spec: &StationSpec<'_>);

    /// Text attached to a station - name under the disc, number
    /// inside it, score further out.
    fn draw_label(&self, ctx: &Context, spec: &LabelSpec<'_>);

    /// The anchor marker (filled = tracked, hollow = virtual).
    fn draw_anchor(&self, ctx: &Context, pos: PxPos, tracked: bool, px_per_mm: PixelsPerMm);

    /// The overlay hint badge (tower missing etc) - `bottom`
    /// reserves room for edge UI the badge must not cover.
    fn draw_hint(&self, ctx: &Context, text: &str, width: Pixels, height: Pixels, bottom: Pixels);
}
