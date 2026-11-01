//! Text content replacement declaration.

use serde::Deserialize;

/// One character data replacement on a parametrized `<text>` element.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextFill {
    /// Id of the parametrized `<text>` element in the master.
    pub id: String,
    /// Character data written into the element.
    pub content: String,
}
