//! Inner markup replacement declaration.

use serde::Deserialize;

/// One inner markup replacement on a parametrized container element.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerFill {
    /// Id of the parametrized container element in the master.
    pub id: String,
    /// SVG markup written into the element (for example glyph paths).
    pub markup: String,
}
