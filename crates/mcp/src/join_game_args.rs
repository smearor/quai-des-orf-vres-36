//! `JoinGameArgs` - parameters of the `join_game` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::RoleClaim;

/// The seat an agent claims - `"fugitive"` or
/// `{"detective": {"seat": 2}}`.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct JoinGameArgs {
    /// The role claim.
    pub claim: RoleClaim,
}
