//! Root type of the variants configuration.

use crate::model::Asset;
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
    /// The asset families declared in the configuration (`[[asset]]`).
    #[serde(rename = "asset")]
    pub assets: Vec<Asset>,
}
