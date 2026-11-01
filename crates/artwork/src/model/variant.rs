//! Single variant declaration.

use crate::model::InnerFill;
use crate::model::Substitution;
use crate::model::TextFill;
use serde::Deserialize;

/// One generated variant of an asset family.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    /// Variant name; substituted into the output template as `{name}`.
    pub name: String,
    /// Attribute substitutions on parametrized element ids.
    #[serde(default)]
    pub substitutions: Vec<Substitution>,
    /// Inner markup replacements on parametrized container elements.
    #[serde(default)]
    pub fills: Vec<InnerFill>,
    /// Character data replacements on parametrized `<text>` elements.
    #[serde(default)]
    pub texts: Vec<TextFill>,
}
