//! `StyleName` - the artwork style a `set_style` command selects.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// The map artwork style. Closed vocabulary: serde rejects
/// unknown spellings at the tool boundary, so the app-side
/// `SetStyle` handler never sees an invalid name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum StyleName {
    /// Flat fills, bold outlines, halftone water (default).
    Comic,
    /// Sepia hatching, serif labels - Belle Époque.
    #[serde(rename = "belle_epoque", alias = "belle-epoque", alias = "belleepoque")]
    BelleEpoque,
    /// Dark field + glow strokes - Neon.
    Neon,
}
