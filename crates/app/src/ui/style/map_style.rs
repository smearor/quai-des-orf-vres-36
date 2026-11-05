//! `MapStyle` - which look the scene renders in (`ARTWORK.md` §2).

use crate::ui::style::BelleEpoqueStyle;
use crate::ui::style::ComicStyle;
use crate::ui::style::NeonStyle;
use crate::ui::style::StyleRenderer;
use qdo36_mcp::StyleName;
use serde::Deserialize;
use serde::Serialize;

/// The map look. `Comic` is the default; the other styles are seams
/// for `ARTWORK.md` §2's palettes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapStyle {
    /// Flat fills, bold outlines, halftone water (default).
    #[default]
    Comic,
    /// Sepia hatching, serif labels - Belle Époque.
    BelleEpoque,
    /// Dark field + glow strokes - Neon.
    Neon,
}

impl MapStyle {
    /// The artwork namespace (`art/{family}/paris/{namespace}/`).
    pub fn namespace(&self) -> &'static str {
        match self {
            Self::Comic => "comic",
            Self::BelleEpoque => "belle-epoque",
            Self::Neon => "neon",
        }
    }

    /// The renderer for this style.
    pub fn renderer(&self) -> Box<dyn StyleRenderer> {
        match self {
            Self::Comic => Box::new(ComicStyle),
            Self::BelleEpoque => Box::new(BelleEpoqueStyle),
            Self::Neon => Box::new(NeonStyle),
        }
    }
}

impl From<StyleName> for MapStyle {
    fn from(name: StyleName) -> Self {
        match name {
            StyleName::Comic => Self::Comic,
            StyleName::BelleEpoque => Self::BelleEpoque,
            StyleName::Neon => Self::Neon,
        }
    }
}
