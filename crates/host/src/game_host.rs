//! `GameHost` - the owner of the live [`GameSession`].
//!
//! MCP commands arrive over an `async_channel`, get executed here
//! (on the app's glib main loop, or a headless runner's thread) and
//! replied via oneshot - physical and agent inputs share
//! `request_move` (WP05 R3/R5). The host is deliberately GTK-free
//! and `Send`-compatible: the map is held behind `Arc`.

use qdo36_map::GameMap;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_mcp::CommandKind;
use qdo36_mcp::CommandReply;
use qdo36_mcp::CommandResult;
use qdo36_mcp::GameStateView;
use qdo36_mcp::McpError;
use qdo36_mcp::RoleClaim;
use qdo36_moves::GamePhase;
use qdo36_moves::MoveRequest;
use qdo36_session::GameConfig;
use qdo36_session::GameSession;
use qdo36_session::PlayerToken;
use qdo36_session::RevealPreset;
use rand::Rng;
use rand::rngs::StdRng;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Owns the running session, the seat↔token registry and the map the
/// rules engine validates against.
pub struct GameHost {
    /// The dealt game, if any.
    session: Option<GameSession>,
    /// The loaded city map (set at scene load).
    map: Option<Arc<GameMap>>,
    /// Issued agent credentials → claimed seat.
    players: BTreeMap<PlayerToken, Role>,
    /// Deal/token draw source.
    rng: StdRng,
}

impl GameHost {
    /// A host with the given draw source (`StdRng::seed_from_u64` in
    /// tests, `from_rng` in the app).
    pub fn with_rng(rng: StdRng) -> Self {
        Self {
            session: None,
            map: None,
            players: BTreeMap::new(),
            rng,
        }
    }

    /// The map moves are validated against (`Arc` keeps the host
    /// `Send` for headless drains).
    pub fn set_map(&mut self, map: Arc<GameMap>) {
        self.map = Some(map);
    }

    /// The session snapshot to publish (clone for the watch doc).
    pub fn session_cloned(&self) -> Option<GameSession> {
        self.session.clone()
    }

    /// Executes one command; the reply type is shared with the MCP
    /// crate so tools serialize it verbatim.
    pub fn handle(&mut self, command: &CommandKind) -> CommandReply {
        match command {
            CommandKind::NewGame { config } => self.new_game(config.clone()),
            CommandKind::Begin => self.begin(),
            CommandKind::SetConfig { reveal, max_rounds } => self.set_config(reveal.clone(), *max_rounds),
            CommandKind::Join { claim } => self.join(*claim),
            CommandKind::GameState { token } => self.view(*token),
            CommandKind::LegalMoves { token } => self.legal(*token),
            CommandKind::Move { token, request } => self.play(*token, *request),
            // The app-side drain intercepts screenshots; a host never
            // renders windows.
            CommandKind::Screenshot => Err(McpError::InvalidParams("screenshot is an app command".to_string())),
        }
    }

    /// Deals a fresh game; `None` takes the map's defaults (city
    /// stocks, reveal rounds, max rounds). Resets the token registry.
    fn new_game(&mut self, config: Option<GameConfig>) -> CommandReply {
        let map = Arc::clone(self.map.as_ref().ok_or(McpError::StateUnavailable)?);
        let config = config.unwrap_or_else(|| GameConfig::for_map(map.meta()));
        config.validate()?;
        let session = GameSession::deal(config, &map, &mut self.rng)?;
        self.players.clear();
        self.session = Some(session);
        Ok(CommandResult::GameDealt)
    }

    /// Leaves the Deal phase.
    fn begin(&mut self) -> CommandReply {
        self.session_mut()?.begin()?;
        Ok(CommandResult::GameBegan)
    }

    /// Adjusts config fields while in the Deal phase.
    fn set_config(&mut self, reveal: Option<RevealPreset>, max_rounds: Option<RoundNumber>) -> CommandReply {
        let session = self.session_mut()?;
        if session.state.phase != GamePhase::Deal {
            return Err(McpError::DealPhaseOnly);
        }
        if let Some(preset) = reveal {
            session.config.reveal = preset;
        }
        if let Some(rounds) = max_rounds {
            session.config.max_rounds = rounds;
        }
        session.config.validate()?;
        Ok(CommandResult::ConfigApplied(session.config.clone()))
    }

    /// Issues a token for a role claim; a seat is taken once.
    fn join(&mut self, claim: RoleClaim) -> CommandReply {
        let session = self.session.as_ref().ok_or(McpError::NoActiveGame)?;
        let role = claim.to_role();
        if let RoleClaim::Detective { seat } = claim
            && !session.state.positions.contains_key(&seat)
        {
            return Err(McpError::SeatUnavailable);
        }
        if self.players.values().any(|taken| *taken == role) {
            return Err(McpError::RoleTaken(role.to_string()));
        }
        let token = PlayerToken::new(self.rng.next_u64());
        self.players.insert(token, role);
        Ok(CommandResult::Joined { token, role })
    }

    /// The token's role, or `BadToken`.
    fn seat_of(&self, token: PlayerToken) -> Result<Role, McpError> {
        self.players.get(&token).copied().ok_or(McpError::BadToken)
    }

    /// Role-scoped state; the Fugitive token additionally sees the
    /// hidden position (WP05 R4).
    fn view(&self, token: PlayerToken) -> CommandReply {
        let role = self.seat_of(token)?;
        let session = self.session.as_ref().ok_or(McpError::NoActiveGame)?;
        let view = GameStateView {
            phase: session.state.phase,
            round: session.state.round,
            your_role: role,
            positions: session.state.positions.clone(),
            fugitive_pos: (role == Role::Fugitive).then_some(session.state.fugitive_pos),
            your_stock: session.state.tickets.get(&role).copied().unwrap_or_default(),
            log: session.state.log.fugitive_public_view(&session.config.reveal_schedule()),
            outcome: session.state.outcome,
            can_undo: session.can_undo(),
            can_redo: session.can_redo(),
        };
        Ok(CommandResult::State(Box::new(view)))
    }

    /// Legal destinations for the token's role.
    fn legal(&self, token: PlayerToken) -> CommandReply {
        let role = self.seat_of(token)?;
        let session = self.session.as_ref().ok_or(McpError::NoActiveGame)?;
        let map = self.map.as_ref().ok_or(McpError::StateUnavailable)?;
        Ok(CommandResult::Legal(session.legal_destinations(map, role)))
    }

    /// A move for the token's seat - `actor` is bound to the token's
    /// role so agents cannot impersonate other seats.
    fn play(&mut self, token: PlayerToken, mut request: MoveRequest) -> CommandReply {
        let role = self.seat_of(token)?;
        let map = Arc::clone(self.map.as_ref().ok_or(McpError::StateUnavailable)?);
        request.actor = role;
        let outcome = self.session_mut()?.request_move(&map, &request)?;
        Ok(CommandResult::Moved(outcome))
    }

    fn session_mut(&mut self) -> Result<&mut GameSession, McpError> {
        self.session.as_mut().ok_or(McpError::NoActiveGame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qdo36_material::PawnId;
    use qdo36_material::TicketKind;
    use qdo36_mcp::RoleClaim;
    use qdo36_moves::MoveKind;
    use rand::SeedableRng;

    fn host() -> GameHost {
        let map = GameMap::bundled().unwrap_or_else(|_| panic!("map"));
        let mut host = GameHost::with_rng(StdRng::seed_from_u64(36));
        host.set_map(Arc::new(map));
        host
    }

    fn dealt_host() -> GameHost {
        let mut host = host();
        host.handle(&CommandKind::NewGame { config: None }).unwrap_or_else(|_| panic!("deal"));
        host
    }

    fn join(host: &mut GameHost, claim: RoleClaim) -> PlayerToken {
        match host.handle(&CommandKind::Join { claim }).unwrap_or_else(|_| panic!("join")) {
            CommandResult::Joined { token, .. } => token,
            _ => panic!("expected Joined"),
        }
    }

    #[test]
    fn join_requires_a_seat_and_is_unique() {
        let mut host = dealt_host();
        let token = join(&mut host, RoleClaim::Fugitive);
        let err = host.handle(&CommandKind::Join { claim: RoleClaim::Fugitive }).err().unwrap_or_else(|| panic!());
        assert!(matches!(err, McpError::RoleTaken(_)), "a role seats once: {token}");
        let err = host
            .handle(&CommandKind::Join {
                claim: RoleClaim::Detective { seat: PawnId::new(99) },
            })
            .err()
            .unwrap_or_else(|| panic!());
        assert!(matches!(err, McpError::SeatUnavailable));
    }

    #[test]
    fn fugitive_token_sees_the_hidden_position() {
        let mut host = dealt_host();
        let fugitive = join(&mut host, RoleClaim::Fugitive);
        let detective = join(&mut host, RoleClaim::Detective { seat: PawnId::new(0) });
        let CommandResult::State(view) = host.handle(&CommandKind::GameState { token: fugitive }).unwrap_or_else(|_| panic!()) else {
            panic!("state")
        };
        assert!(view.fugitive_pos.is_some(), "the fugitive token sees his position");
        let CommandResult::State(view) = host.handle(&CommandKind::GameState { token: detective }).unwrap_or_else(|_| panic!()) else {
            panic!("state")
        };
        assert!(view.fugitive_pos.is_none(), "detectives never see the position");
    }

    #[test]
    fn move_binds_actor_to_the_token_role() {
        let mut host = dealt_host();
        host.handle(&CommandKind::Begin).unwrap_or_else(|_| panic!("begin"));
        let detective = join(&mut host, RoleClaim::Detective { seat: PawnId::new(0) });
        let session = host.session.as_ref().unwrap_or_else(|| panic!("session"));
        let map = host.map.clone().unwrap_or_else(|| panic!("map"));
        let hop = session.legal_destinations(&map, Role::Fugitive)[0];
        // The detective token tries a Fugitive hop; the host binds the
        // actor to the detective so validation rejects the request as
        // NotPlayersTurn, not as impersonation.
        let result = host.handle(&CommandKind::Move {
            token: detective,
            request: MoveRequest {
                actor: Role::Fugitive,
                ticket: TicketKind::Taxi,
                kind: MoveKind::Single(hop),
            },
        });
        assert!(result.is_err(), "the token can never play as fugitive");
    }

    #[test]
    fn set_config_is_deal_phase_only() {
        let mut host = dealt_host();
        host.handle(&CommandKind::SetConfig {
            reveal: None,
            max_rounds: Some(RoundNumber::new(30)),
        })
        .unwrap_or_else(|_| panic!("set_config"));
        assert_eq!(host.session.as_ref().unwrap_or_else(|| panic!()).config.max_rounds.number(), 30);
        host.handle(&CommandKind::Begin).unwrap_or_else(|_| panic!("begin"));
        let err = host
            .handle(&CommandKind::SetConfig {
                reveal: None,
                max_rounds: Some(RoundNumber::new(30)),
            })
            .err()
            .unwrap_or_else(|| panic!());
        assert!(matches!(err, McpError::DealPhaseOnly));
    }
}
