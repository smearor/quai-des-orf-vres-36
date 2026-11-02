//! `RoleClaim` - the seat an agent asks for in `join_game`.

use qdo36_material::PawnId;
use qdo36_material::Role;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// The role an agent claims when joining.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RoleClaim {
    /// The hidden player - requires the Corbeau authority in a real
    /// deployment (LAN trust model, WP05 R4).
    Fugitive,
    /// A detective seat by pawn index.
    Detective {
        /// The pawn/seat to play.
        #[schemars(with = "u8")]
        seat: PawnId,
    },
}

impl RoleClaim {
    /// The [`Role`] the claim maps to.
    pub fn to_role(self) -> Role {
        match self {
            Self::Fugitive => Role::Fugitive,
            Self::Detective { seat } => Role::Detective(seat),
        }
    }
}
