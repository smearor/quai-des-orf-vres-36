//! The `#[tool_router]` surface: introspection tools reading the
//! shared state snapshot plus the agent play API that round-trips
//! [`CommandKind`]s through the app's session owner.

use super::qdo_mcp::QdoMcp;
use crate::CommandKind;
use crate::JoinGameArgs;
use crate::McpError;
use crate::NewGameArgs;
use crate::RequestMoveArgs;
use crate::SetConfigArgs;
use crate::TokenArgs;
use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;

#[tool_router(vis = "pub(super)")]
impl QdoMcp {
    /// Live anchor facts: detection flag, centroid and orientation.
    #[tool(description = "Get the anchor state: whether the Eiffel tower is detected, its centroid in device mm and its orientation in degrees.")]
    async fn anchor_state(&self) -> Result<CallToolResult, ErrorData> {
        self.reply(&self.snapshot().anchor)
    }

    /// Static facts of the loaded city map.
    #[tool(description = "Get map info: city name, station/edge counts, map bounds in mm and the anchor station position.")]
    async fn map_info(&self) -> Result<CallToolResult, ErrorData> {
        let info = self.snapshot().map.ok_or(McpError::StateUnavailable)?;
        self.reply(&info)
    }

    /// The public move log - exactly what the HUD shows: detective
    /// destinations are public, Fugitive's are `null` until their
    /// reveal round.
    #[tool(description = "Get the public move log: detective moves incl. destination, Corbeau entries ticket-only until their reveal round.")]
    async fn move_log(&self) -> Result<CallToolResult, ErrorData> {
        let session = self.snapshot().game.ok_or(McpError::NoActiveGame)?;
        let view = session.state.log.fugitive_public_view(&session.config.reveal_schedule());
        self.reply(&view)
    }

    /// The effective game config: mode, reveal preset + resolved
    /// schedule, max_rounds, allow_undo and the ticket stocks.
    #[tool(description = "Get the game config: mode, reveal preset and resolved schedule, max_rounds, allow_undo and starting ticket stocks.")]
    async fn game_config(&self) -> Result<CallToolResult, ErrorData> {
        let session = self.snapshot().game.ok_or(McpError::NoActiveGame)?;
        self.reply(&session.config)
    }

    /// Deals a fresh game (Deal phase; `set_config` stays legal until
    /// `begin_game` is called).
    #[tool(
        description = "Deal a new game: samples start stations, seats the detectives and hides Corbeau. Optional 'config' overrides the map defaults; call set_config for reveal/max_rounds, then begin_game."
    )]
    async fn new_game(&self, Parameters(args): Parameters<NewGameArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::NewGame { config: args.config }).await
    }

    /// Adjusts reveal schedule and game length pre-game.
    #[tool(description = "Pre-game config: reveal preset ('classic'/'none' or custom 'rounds') and max_rounds. Deal phase only.")]
    async fn set_config(&self, Parameters(args): Parameters<SetConfigArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::SetConfig {
            reveal: args.reveal,
            max_rounds: args.max_rounds,
        })
        .await
    }

    /// Leaves the Deal phase - the turn cycle opens with Fugitive.
    #[tool(description = "Begin the game: leaves the Deal phase; moves are accepted afterwards.")]
    async fn begin_game(&self) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::Begin).await
    }

    /// Seats an agent under a role claim; returns the issued
    /// `PlayerToken` scoping all later play calls.
    #[tool(description = "Join the game as 'fugitive' or 'detective' (with seat). Returns the player token scoping all later calls.")]
    async fn join_game(&self, Parameters(args): Parameters<JoinGameArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::Join { claim: args.claim }).await
    }

    /// The role-scoped game view: public projection for detectives,
    /// plus the hidden position for the Fugitive token.
    #[tool(description = "Get the game state scoped to your token: phase, round, seats, public log; the fugitive token additionally sees the hidden position.")]
    async fn game_state(&self, Parameters(args): Parameters<TokenArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::GameState { token: args.token }).await
    }

    /// The destinations the token's role may reach this turn.
    #[tool(description = "List the legal destinations for your token's role this turn.")]
    async fn legal_moves(&self, Parameters(args): Parameters<TokenArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::LegalMoves { token: args.token }).await
    }

    /// Submits a move for the token's seat - identical validation to
    /// physical input (pawn/tap), same `GameSession::request_move`.
    #[tool(description = "Request a move for your token's seat: one hop, or two with the 'double' ticket. Same validation as physical input.")]
    async fn request_move(&self, Parameters(args): Parameters<RequestMoveArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::Move {
            token: args.token,
            request: args.request(),
        })
        .await
    }
}

impl QdoMcp {
    /// Forwards a command to the app-side game host and serializes
    /// the typed reply.
    async fn forward(&self, kind: CommandKind) -> Result<CallToolResult, ErrorData> {
        let result = self.command(kind).await?;
        self.reply(&result)
    }

    /// Serializes a reply as pretty JSON text content.
    fn reply(&self, value: &impl serde::Serialize) -> Result<CallToolResult, ErrorData> {
        let text = serde_json::to_string_pretty(value).map_err(McpError::from)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
    }
}
