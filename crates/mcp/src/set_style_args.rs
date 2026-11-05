//! `SetStyleArgs` - parameters of the `set_style` tool.

use crate::StyleName;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Switch the map artwork style at runtime.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetStyleArgs {
    /// One of `"comic"`, `"belle_epoque"`, `"neon"`.
    pub style: StyleName,
}
