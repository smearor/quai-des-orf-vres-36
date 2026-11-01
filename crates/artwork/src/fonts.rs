//! Bundled font registry used by the text-outlining pass.
//!
//! Loads every TTF found below the shared `resources/fonts` directory into
//! a [`fontdb::Database`] and resolves CSS-style `font-family` stacks plus
//! `font-weight` values to concrete faces for shaping.

use crate::error::ArtworkError;
use std::path::Path;

/// Registry of bundled OpenType faces resolved by family name and weight.
pub(crate) struct FontRegistry {
    db: fontdb::Database,
}

impl FontRegistry {
    /// Loads every font file found below `fonts_dir` (a missing directory
    /// yields an empty registry, which later fails per family).
    pub(crate) fn load(fonts_dir: &Path) -> Self {
        let mut db = fontdb::Database::new();
        db.load_fonts_dir(fonts_dir);
        Self { db }
    }

    /// Resolves the first available family in `family_list` at `weight` to
    /// a face identifier. Generic families (`sans-serif`, `serif`,
    /// `monospace`) are honoured as last-resort fallbacks.
    pub(crate) fn face_id(&self, family_list: &str, weight: u16) -> Result<fontdb::ID, ArtworkError> {
        let family_list = family_list.replace("&#39;", "'").replace("&quot;", "\"");
        for family_name in family_list.split(',') {
            let name = family_name.trim().trim_matches(|c| c == '\'' || c == '"');
            if name.is_empty() {
                continue;
            }
            let family = match name {
                "sans-serif" => fontdb::Family::SansSerif,
                "serif" => fontdb::Family::Serif,
                "monospace" => fontdb::Family::Monospace,
                "cursive" => fontdb::Family::Cursive,
                other => fontdb::Family::Name(other),
            };
            let query = fontdb::Query {
                families: &[family],
                weight: fontdb::Weight(weight),
                ..fontdb::Query::default()
            };
            if let Some(id) = self.db.query(&query) {
                return Ok(id);
            }
        }
        Err(ArtworkError::MissingFontFamily { family: family_list })
    }

    /// Runs `f` with the raw face data of `id`; the data only lives for the
    /// callback's duration, so every font operation (shaping, glyph
    /// extraction) must happen inside `f`.
    pub(crate) fn with_face_data<R>(&self, id: fontdb::ID, f: impl FnOnce(&[u8], u32) -> R) -> Result<R, ArtworkError> {
        self.db
            .with_face_data(id, f)
            .ok_or_else(|| ArtworkError::MissingFontFamily { family: format!("id {id}") })
    }
}
