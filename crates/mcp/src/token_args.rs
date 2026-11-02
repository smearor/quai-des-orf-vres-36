//! `TokenArgs` - parameters of token-scoped tools.

use qdo36_session::PlayerToken;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// A player token issued by `join_game` (16 hex digits).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TokenArgs {
    /// The player's token.
    #[schemars(with = "String")]
    pub token: PlayerToken,
}
