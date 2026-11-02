//! The generated SVG artwork (`resources/art/`): cached rsvg
//! handles, name-to-stem mapping, and the draw spec.
//!
//! `resources/art/{family}/{city}/{style}/` holds the font-free SVGs
//! produced by `qdo36-artwork`. Handles are loaded lazily and cached
//! per path; missing files resolve to `None` so callers fall back to
//! the `StyleRenderer` primitives.

pub mod anchor;
pub mod art_assets;
pub mod asset_stem;
pub mod doc_size;
pub mod family;
pub mod spec;

pub use anchor::AnchorFrac;
pub use art_assets::ArtAssets;
pub use asset_stem::AssetStem;
pub use doc_size::DocumentSize;
pub use family::ArtFamily;
pub use spec::ArtSpec;

#[cfg(test)]
mod tests {
    use cairo::{Context, Format, ImageSurface};
    use super::*;
    use crate::ui::style::Alpha;
    use crate::ui::style::MapStyle;
    use crate::ui::style::PxPos;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::PixelsPerMm;

    #[test]
    fn draws_existing_assets_and_falls_back() {
        let assets = ArtAssets::detect();
        let surface = ImageSurface::create(Format::ARgb32, 64, 64);
        let Ok(surface) = surface else { return };
        let Ok(ctx) = Context::new(&surface) else { return };
        let px_per_mm = PixelsPerMm::new(2.0);
        let pos = PxPos::new(32.0, 32.0);

        let spec = ArtSpec {
            family: ArtFamily::Landmarks,
            stem: AssetStem::new("louvre"),
            pos,
            height: Millimeters::new(12.0),
            anchor: AnchorFrac::PIN_TIP,
            alpha: Alpha::OPAQUE,
        };
        assert!(assets.draw(&ctx, MapStyle::BelleEpoque, &spec, px_per_mm));

        let missing = ArtSpec {
            family: ArtFamily::Landmarks,
            stem: AssetStem::new("does-not-exist"),
            pos,
            height: Millimeters::new(12.0),
            anchor: AnchorFrac::PIN_TIP,
            alpha: Alpha::OPAQUE,
        };
        assert!(!assets.draw(&ctx, MapStyle::Comic, &missing, px_per_mm));
    }
}
