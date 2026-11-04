//! The `#[tool_router]` surface: introspection tools reading the
//! shared state snapshot plus the agent play API that round-trips
//! [`CommandKind`]s through the app's session owner.

use super::qdo_mcp::QdoMcp;
use crate::CommandKind;
use crate::CommandResult;
use crate::JoinGameArgs;
use crate::McpError;
use crate::NewGameArgs;
use crate::RequestMoveArgs;
use crate::SetAnchorArgs;
use crate::SetConfigArgs;
use crate::SetLayersArgs;
use crate::SetParamsArgs;
use crate::SetViewArgs;
use crate::TokenArgs;
use qdo36_tangibles::MapScale;
use qdo36_tangibles::ScaleFactor;
use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::RotationRad;
use tangible_objects_model::Vec2;

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

    /// PNG screenshot of the app window.
    #[tool(description = "Take a screenshot of the app window; returns a PNG image.")]
    async fn screenshot(&self) -> Result<CallToolResult, ErrorData> {
        let result = self.command(CommandKind::Screenshot).await??;
        let CommandResult::Screenshot(reply) = result else {
            return Err(McpError::InvalidParams("unexpected reply kind for screenshot".to_string()).into());
        };
        Ok(CallToolResult::success(vec![
            ContentBlock::image(reply.png_base64, "image/png".to_string()),
            ContentBlock::text(format!("screenshot ({:.0}x{:.0})", reply.width.as_f64(), reply.height.as_f64())),
        ]))
    }

    /// Lint the loaded map dataset against the quality criteria
    /// (planarity, connectivity, line linearity, coverage, density).
    #[tool(
        description = "Lint the map dataset: crossings per layer, connectivity per transport, metro-line linearity + transfers, service coverage, density metrics."
    )]
    async fn map_lint(&self) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::MapLint).await
    }

    /// Every drawn entity's mm + px position under the current view.
    #[tool(description = "Get the scene layout: every station and landmark with its position in map-mm and widget-px under the current view transform.")]
    async fn scene_layout(&self) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::SceneLayout).await
    }

    /// Pan/rotate/zoom the view for visual debugging (view-only,
    /// not persisted).
    #[tool(description = "Set the view: 'zoom' factor (0.5-5.0), 'rotate' in quarter turns (0-3), 'pan_mm' [dx,dy] in surface-mm.")]
    async fn set_view(&self, Parameters(args): Parameters<SetViewArgs>) -> Result<CallToolResult, ErrorData> {
        let zoom = args
            .zoom
            .map(MapScale::try_new)
            .transpose()
            .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
        let rotate = args
            .rotate
            .map(|turns| match turns {
                0 => Ok(DisplayRotation::Deg0),
                1 => Ok(DisplayRotation::Deg90),
                2 => Ok(DisplayRotation::Deg180),
                3 => Ok(DisplayRotation::Deg270),
                other => Err(ErrorData::invalid_params(format!("rotate must be 0-3, got {other}"), None)),
            })
            .transpose()?;
        self.forward(CommandKind::SetView {
            zoom,
            rotate,
            pan: args.pan_mm.map(|[dx, dy]| Vec2::new(dx, dy)),
            compass: args.compass_mm.map(|[x, y]| Vec2::new(x, y)),
        })
        .await
    }

    /// Reload the city dataset from disk - iterate map data
    /// without restarting the app.
    #[tool(description = "Reload the city map dataset from disk (dev iteration, no restart).")]
    async fn reload_map(&self) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::ReloadMap).await
    }

    /// Adjust the global render factors (1.0 neutral): station discs,
    /// landmark pins, metro badges, transport rings, label fonts.
    #[tool(description = "Adjust global render factors (station/landmark/metro sizes, ring width, label font).")]
    async fn set_params(&self, Parameters(args): Parameters<SetParamsArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::SetParams {
            station_scale: args.station_scale.map(ScaleFactor::new),
            landmark_scale: args.landmark_scale.map(ScaleFactor::new),
            metro_scale: args.metro_scale.map(ScaleFactor::new),
            ring_width: args.ring_width.map(ScaleFactor::new),
            label_scale: args.label_scale.map(ScaleFactor::new),
        })
        .await
    }

    /// Set a virtual anchor pose on the surface - bypasses the
    /// "place the tower" setup for headless/table-less runs.
    #[tool(description = "Set a virtual anchor: 'center_mm' [x,y] in surface-mm and optional 'rotation_rad' - places the tower without a tangible.")]
    async fn set_anchor(&self, Parameters(args): Parameters<SetAnchorArgs>) -> Result<CallToolResult, ErrorData> {
        let [x, y] = args.center_mm;
        self.forward(CommandKind::SetAnchor {
            center: Vec2::new(x, y),
            rotation_rad: args.rotation_rad.map(RotationRad::new),
        })
        .await
    }

    /// Toggle scene layers (water, landmarks, per-transport edges,
    /// stations, labels) for visual debugging - plus `hide_lines`
    /// to hide single metro lines by name.
    #[tool(
        description = "Toggle scene layers: water, landmarks, taxi/bus/metro/batobus/underground edges, stations, labels - plus 'hide_lines': [\"M4\",\"M13\"] to hide single metro lines."
    )]
    async fn set_layers(&self, Parameters(args): Parameters<SetLayersArgs>) -> Result<CallToolResult, ErrorData> {
        self.forward(CommandKind::SetLayers { mask: args.mask }).await
    }
}

impl QdoMcp {
    /// Forwards a command to the app-side game host and serializes
    /// the `CommandResult` - a domain failure becomes a tool-level
    /// `isError` reply instead of an `{"Err": ...}` payload.
    async fn forward(&self, kind: CommandKind) -> Result<CallToolResult, ErrorData> {
        let result = self.command(kind).await??;
        self.reply(&result)
    }

    /// Serializes a reply as pretty JSON text content.
    fn reply(&self, value: &impl serde::Serialize) -> Result<CallToolResult, ErrorData> {
        let text = serde_json::to_string_pretty(value).map_err(McpError::from)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
    }
}
