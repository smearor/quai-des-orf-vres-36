//! `StyleRenderer` - the draw interface a map style implements.

use cairo::Context;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;
use tangible_objects_model::Vec2;

use crate::ui::style::EdgeSpec;
use crate::ui::style::RgbColor;

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

    /// A water polyline (river, canal).
    fn draw_water(&self, ctx: &Context, points: &[Vec2<Millimeters>], px_per_mm: PixelsPerMm, to_px: &dyn Fn(Vec2<Millimeters>) -> PxPos);

    /// A station pair's edge.
    fn draw_edge(&self, ctx: &Context, edge: &EdgeSpec<'_>);

    /// A backdrop landmark marker.
    fn draw_landmark(&self, ctx: &Context, pos: PxPos, px_per_mm: PixelsPerMm);

    /// A station node (fill + outline + selection ring).
    fn draw_station(&self, ctx: &Context, pos: PxPos, selected: bool, px_per_mm: PixelsPerMm);

    /// A station name label below the node.
    fn draw_label(&self, ctx: &Context, name: &str, pos: PxPos, radius: Pixels, px_per_mm: PixelsPerMm);

    /// The anchor marker (filled = tracked, hollow = virtual).
    fn draw_anchor(&self, ctx: &Context, pos: PxPos, tracked: bool, px_per_mm: PixelsPerMm);

    /// The overlay hint badge (tower missing etc).
    fn draw_hint(&self, ctx: &Context, text: &str, width: Pixels, height: Pixels);
}
