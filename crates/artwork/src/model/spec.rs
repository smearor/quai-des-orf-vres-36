//! Root type of the variants configuration.

use crate::model::Asset;
use serde::Deserialize;

/// Root of `variants.toml`: the list of asset families to generate.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// The asset families declared in the configuration (`[[asset]]`).
    #[serde(rename = "asset")]
    pub assets: Vec<Asset>,
}
