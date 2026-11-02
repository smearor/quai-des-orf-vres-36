//! `NewGameArgs` - parameters of the `new_game` tool.

use qdo36_session::GameConfig;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Optional setup for `new_game`; empty args deal the map's defaults.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct NewGameArgs {
    /// Explicit game setup - needed for deal-shaping fields like
    /// detective count, ticket stocks or `allow_undo`. Omit to use
    /// the loaded map's defaults.
    #[serde(default)]
    #[schemars(with = "serde_json::Value")]
    pub config: Option<GameConfig>,
}
