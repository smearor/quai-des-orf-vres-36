//! The generated SVG artwork (`resources/art/`): cached rsvg
//! handles, name-to-stem mapping, and the draw spec.
//!
//! `resources/art/{family}/{city}/{style}/` holds the font-free SVGs
//! produced by `qdo36-artwork`. Handles are loaded lazily and cached
//! per path; missing files resolve to `None` so callers fall back to
//! the `StyleRenderer` primitives.

pub mod anchor;
pub mod art_assets;
pub mod spec;

pub use anchor::AnchorFrac;
pub use art_assets::ArtAssets;
pub use spec::ArtSpec;

/// The asset stem for a display name: lowercased, de-accented,
/// punctuation turned into dashes.
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|ch| match ch {
            'à' | 'â' | 'ä' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            other => other,
        })
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Landmarks whose asset stem does not follow from `slugify`
/// (renamed artwork, reordered words).
const LANDMARK_ALIASES: &[(&str, &str)] = &[
    ("tour-eiffel", "eiffel-tour"),
    ("musee-du-louvre", "louvre"),
    ("place-de-la-concorde", "place-concorde"),
    ("place-de-la-bastille", "place-bastille"),
    ("place-de-la-republique", "place-republique"),
    ("la-seine", "seine-boat"),
];

/// The asset stem for a landmark display name.
pub fn landmark_stem(name: &str) -> String {
    let slug = slugify(name);
    LANDMARK_ALIASES
        .iter()
        .find_map(|(from, to)| (slug == *from).then_some(*to))
        .unwrap_or(&slug)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::style::Alpha;
    use crate::ui::style::MapStyle;
    use crate::ui::style::PxPos;
    use tangible_objects_model::Millimeters;
    use tangible_objects_model::PixelsPerMm;

    #[test]
    fn slugifies_display_names() {
        assert_eq!(slugify("Arc de Triomphe"), "arc-de-triomphe");
        assert_eq!(slugify("Panthéon"), "pantheon");
        assert_eq!(slugify("Gare du Nord"), "gare-du-nord");
        assert_eq!(slugify("Île de la Cité"), "ile-de-la-cite");
    }

    #[test]
    fn aliases_landmarks() {
        assert_eq!(landmark_stem("Tour Eiffel"), "eiffel-tour");
        assert_eq!(landmark_stem("Musée du Louvre"), "louvre");
        assert_eq!(landmark_stem("Place de la Concorde"), "place-concorde");
        assert_eq!(landmark_stem("Louvre"), "louvre");
    }

    #[test]
    fn draws_existing_assets_and_falls_back() {
        let assets = ArtAssets::detect();
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 64, 64);
        let Ok(surface) = surface else { return };
        let Ok(ctx) = cairo::Context::new(&surface) else { return };
        let px_per_mm = PixelsPerMm::new(2.0);
        let pos = PxPos::new(32.0, 32.0);

        let spec = ArtSpec {
            family: "landmarks",
            stem: "louvre",
            pos,
            height: Millimeters::new(12.0),
            anchor: AnchorFrac::PIN_TIP,
            alpha: Alpha::OPAQUE,
        };
        assert!(assets.draw(&ctx, MapStyle::BelleEpoque, &spec, px_per_mm));

        let missing = ArtSpec {
            family: "landmarks",
            stem: "does-not-exist",
            pos,
            height: Millimeters::new(12.0),
            anchor: AnchorFrac::PIN_TIP,
            alpha: Alpha::OPAQUE,
        };
        assert!(!assets.draw(&ctx, MapStyle::Comic, &missing, px_per_mm));
    }
}
