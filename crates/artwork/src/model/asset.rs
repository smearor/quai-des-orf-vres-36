//! Asset family declaration.

use crate::model::Variant;
use serde::Deserialize;

/// One asset family: a parametrized master plus the variants to generate.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    /// Master SVG file name inside the masters directory.
    pub master: String,
    /// Output file name template; `{name}` is replaced by the variant name.
    pub output: String,
    /// The variants to generate from this master (`[[asset.variant]]`).
    #[serde(rename = "variant")]
    pub variants: Vec<Variant>,
}
