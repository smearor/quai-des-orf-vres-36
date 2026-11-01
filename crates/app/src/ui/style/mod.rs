//! Swappable map render styles (`ARTWORK.md` §2).
//!
//! `MapScene` owns the scene semantics (what exists: edges, stations,
//! backdrop, anchor); a [`StyleRenderer`] decides how it looks -
//! palette, stroke treatment, decorations, label font.

pub mod alpha;
pub mod color;
pub mod comic;
pub mod edge_spec;
pub mod map_style;
pub mod neon;
pub mod renderer;

pub use alpha::Alpha;
pub use color::RgbColor;
pub use comic::ComicStyle;
pub use edge_spec::EdgeSpec;
pub use map_style::MapStyle;
pub use neon::NeonStyle;
pub use renderer::PxPos;
pub use renderer::StyleRenderer;

use cairo::Context;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Stroke a polyline path in widget px.
pub fn polyline_path(ctx: &Context, points: &[Vec2<Millimeters>], to_px: &dyn Fn(Vec2<Millimeters>) -> PxPos) {
    ctx.new_path();
    for (index, point) in points.iter().enumerate() {
        let pos = to_px(*point);
        if index == 0 {
            ctx.move_to(pos.x, pos.y);
        } else {
            ctx.line_to(pos.x, pos.y);
        }
    }
}
