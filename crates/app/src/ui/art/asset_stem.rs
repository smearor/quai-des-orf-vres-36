//! `AssetStem` - a canonical artwork filename stem.
//!
//! Only values that have been through `slugify` (or a trusted
//! literal) become an `AssetStem`, so `ArtSpec` cannot be handed a
//! raw display name by accident.

use std::fmt;

/// A canonical asset filename stem (no extension, lowercase, dashed).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetStem(String);

impl AssetStem {
    /// A trusted literal stem (already canonical - e.g.
    /// `AssetStem::new("compass")`).
    pub fn new(stem: &str) -> Self {
        Self(stem.to_string())
    }

    /// The stem for a backdrop landmark's display name: slugified,
    /// with a short alias table for artwork that was renamed or
    /// reworded.
    pub fn for_landmark(name: &str) -> Self {
        let slug = slugify(name);
        match slug.as_str() {
            "tour-eiffel" => Self::new("eiffel-tour"),
            "musee-du-louvre" => Self::new("louvre"),
            "place-de-la-concorde" => Self::new("place-concorde"),
            "place-de-la-bastille" => Self::new("place-bastille"),
            "place-de-la-republique" => Self::new("place-republique"),
            "la-seine" => Self::new("seine-boat"),
            _ => Self(slug),
        }
    }

    /// The stem as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AssetStem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Lowercase, de-accent, and fold punctuation into dashes.
fn slugify(name: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_display_names() {
        assert_eq!(AssetStem::new("x").as_str(), "x");
        assert_eq!(AssetStem::for_landmark("Arc de Triomphe").as_str(), "arc-de-triomphe");
        assert_eq!(AssetStem::for_landmark("Panthéon").as_str(), "pantheon");
        assert_eq!(AssetStem::for_landmark("Gare du Nord").as_str(), "gare-du-nord");
        assert_eq!(AssetStem::for_landmark("Île de la Cité").as_str(), "ile-de-la-cite");
    }

    #[test]
    fn aliases_landmarks() {
        assert_eq!(AssetStem::for_landmark("Tour Eiffel").as_str(), "eiffel-tour");
        assert_eq!(AssetStem::for_landmark("Musée du Louvre").as_str(), "louvre");
        assert_eq!(AssetStem::for_landmark("Place de la Concorde").as_str(), "place-concorde");
        assert_eq!(AssetStem::for_landmark("Louvre").as_str(), "louvre");
    }
}
