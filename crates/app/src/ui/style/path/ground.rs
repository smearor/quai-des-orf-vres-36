//! Ground and texture helpers shared by the style renderers.

use crate::ui::style::PxPos;
use crate::ui::style::RgbColor;
use crate::ui::style::ToPx;
use crate::ui::style::path::edge::EdgePathExt;
use cairo::Context;
use cairo::ImageSurface;
use qdo36_map::Ground;

/// Cairo ground fills: `ctx.fill_ground(..)` floods the city's
/// ground mask, `ctx.tile_fill(..)` repeats the outer texture.
pub trait GroundExt {
    /// Fills every ground polygon with `color` - the city rests
    /// on plain paper while the countryside keeps its texture.
    fn fill_ground(&self, ground: &Ground, color: &RgbColor, to_px: ToPx<'_>);
    /// Fills the whole widget with `tile` repeated, anchored so
    /// the tile origin sits at `pin` - the texture repeats
    /// infinitely and pans with the map.
    fn tile_fill(&self, tile: &ImageSurface, pin: &PxPos, width: f64, height: f64);
}

impl GroundExt for Context {
    fn fill_ground(&self, ground: &Ground, color: &RgbColor, to_px: ToPx<'_>) {
        color.apply(self);
        for polygon in ground {
            if polygon.len() < 3 {
                continue;
            }
            let points: Vec<PxPos> = to_px.points(polygon);
            self.px_polyline_path(&points);
            self.close_path();
            self.fill().ok();
        }
    }

    fn tile_fill(&self, tile: &ImageSurface, pin: &PxPos, width: f64, height: f64) {
        let pattern = cairo::SurfacePattern::create(tile);
        pattern.set_extend(cairo::Extend::Repeat);
        pattern.set_matrix(cairo::Matrix::new(1.0, 0.0, 0.0, 1.0, -pin.x, -pin.y));
        self.set_source(&pattern).ok();
        self.rectangle(0.0, 0.0, width, height);
        self.fill().ok();
    }
}
