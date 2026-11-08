//! `OuterSpec` - the countryside outside the city the scene draws.

use qdo36_map::Ground;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;
use tangible_objects_model::Vec2;

/// Everything the outer-texture pass needs: the ground polygons
/// covering the city (everywhere else gets the style's infinite
/// texture) plus the widget size and a map-anchored pin the
/// texture stays put on while panning.
pub struct OuterSpec<'a> {
    /// Ground polygons in map mm - the city area the texture must
    /// not cover (blocks, streets, corridors and suppressed faces
    /// all included).
    pub ground: &'a Ground,
    /// Map position the texture tiles from - infinitely repeated,
    /// anchored to the map so it pans and zooms with the world.
    pub pin: Vec2<Millimeters>,
    /// Widget width in px - the texture covers the whole viewport.
    pub width: Pixels,
    /// Widget height in px.
    pub height: Pixels,
    /// Pixel scale for line widths inside the texture tile.
    pub px_per_mm: PixelsPerMm,
}
