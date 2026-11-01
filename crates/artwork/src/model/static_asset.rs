//! Static asset entry of the variants configuration.

use serde::Deserialize;

/// A static (non-variant) SVG asset processed through the text outline
/// pass. Its `path` is resolved relative to the output directory, read,
/// outlined in place and written back — so hand-authored assets ship
/// font-free like generated ones.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticAsset {
    /// Output-relative path of the static asset; `{city}`/`{style}`
    /// placeholders resolve like variant outputs.
    pub path: String,
}
