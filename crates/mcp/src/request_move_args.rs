//! `RequestMoveArgs` - parameters of the `request_move` tool.

use qdo36_map::StationId;
use qdo36_map::Transport;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::TicketKind;
use qdo36_moves::MoveKind;
use qdo36_moves::MoveRequest;
use qdo36_session::PlayerToken;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::HopArgs;

/// A move request: one hop, plus `second_hop` for the double ticket.
/// `actor` is set by the host from the token - agents cannot
/// impersonate a seat (WP05 R4).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RequestMoveArgs {
    /// The player's token (16 hex digits).
    #[schemars(with = "String")]
    pub token: PlayerToken,
    /// `"taxi"`, `"bus"`, `"metro"`, `"black"` or `"double"`.
    #[schemars(with = "String")]
    pub ticket: TicketKind,
    /// Start station id.
    #[schemars(with = "u16")]
    pub from: StationId,
    /// Transport of the first hop.
    #[schemars(with = "String")]
    pub transport: Transport,
    /// Destination station id.
    #[schemars(with = "u16")]
    pub to: StationId,
    /// The second hop (double ticket only); its `from` is the first
    /// hop's destination.
    #[serde(default)]
    pub second_hop: Option<HopArgs>,
}

impl RequestMoveArgs {
    /// The request, before the host binds `actor` to the token's
    /// role. The placeholder detective is overridden by the host
    /// (agents cannot impersonate a seat).
    pub fn request(&self) -> MoveRequest {
        let first = HopArgs {
            to: self.to,
            transport: self.transport,
        }
        .hop(self.from);
        let kind = match &self.second_hop {
            Some(hop) => MoveKind::Double {
                first,
                second: hop.hop(first.to),
            },
            None => MoveKind::Single(first),
        };
        MoveRequest {
            actor: Role::Detective(PawnId::new(0)),
            ticket: self.ticket,
            kind,
        }
    }
}
