//! Root type of the variants configuration.

use crate::model::Asset;
use crate::model::StaticAsset;
use serde::Deserialize;

/// Root of `variants.toml`: the list of asset families to generate.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// City namespace segment; substituted into output templates as
    /// `{city}` (defaults to `paris`).
    #[serde(default)]
    pub city: Option<String>,
    /// Map style namespace segment; substituted into output templates
    /// as `{style}` (defaults to `belle-epoque`).
    #[serde(default)]
    pub style: Option<String>,
    /// Directory holding the bundled OFL fonts, resolved relative to the
    /// directory containing this configuration file. Required when
    /// `outline` is enabled.
    #[serde(default)]
    pub fonts_dir: Option<String>,
    /// Whether `<text>` elements are converted to outlined glyph paths in
    /// the emitted files (defaults to `true`). Set `outline = false` for
    /// developer previews that keep live, font-dependent text.
    #[serde(default)]
    pub outline: Option<bool>,
    /// The asset families declared in the configuration (`[[asset]]`).
    #[serde(rename = "asset")]
    pub assets: Vec<Asset>,
    /// Static (non-variant) assets processed through the outline pass
    /// (`[[static]]`).
    #[serde(rename = "static", default)]
    pub statics: Vec<StaticAsset>,
}
