//! `HopArgs` - one hop of a `request_move` argument.

use qdo36_map::StationId;
use qdo36_map::Transport;
use qdo36_moves::Hop;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// One hop of a move: `to` via `transport`.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HopArgs {
    /// Destination station id.
    #[schemars(with = "u16")]
    pub to: StationId,
    /// `"taxi"`, `"bus"`, `"metro"`, `"batobus"` or `"underground"`.
    #[schemars(with = "String")]
    pub transport: Transport,
}

impl HopArgs {
    /// A hop starting at `from`.
    pub(crate) fn hop(&self, from: StationId) -> Hop {
        Hop {
            from,
            to: self.to,
            transport: self.transport,
        }
    }
}
