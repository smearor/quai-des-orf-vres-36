//! `SetLayersArgs` - parameters of the `set_layers` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::LayerMask;

/// Layer-visibility toggles for visual debugging.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SetLayersArgs {
    /// Per-layer flags; `null` keeps the current value.
    #[serde(flatten)]
    pub mask: LayerMask,
}
