//! `SetConfigArgs` - parameters of the `set_config` tool.

use qdo36_material::RoundNumber;
use qdo36_session::RevealPreset;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Pre-game config adjustments (Deal phase only).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetConfigArgs {
    /// Reveal preset override - `"classic"`, `"none"` or
    /// `{"custom": {"rounds": [...]}}`; omit to keep.
    #[serde(default)]
    #[schemars(with = "serde_json::Value")]
    pub reveal: Option<RevealPreset>,
    /// Game length override; omit to keep the current value.
    #[serde(default)]
    #[schemars(with = "u8")]
    pub max_rounds: Option<RoundNumber>,
}
