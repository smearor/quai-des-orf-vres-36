//! `CommandKind` - the write commands crossing from MCP tools into
//! the app's session owner.

use qdo36_material::RoundNumber;
use qdo36_moves::MoveRequest;
use qdo36_session::GameConfig;
use qdo36_session::PlayerToken;
use qdo36_session::RevealPreset;
use qdo36_tangibles::MapScale;
use qdo36_tangibles::ScaleFactor;
use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::Millimeters;
use tangible_objects_model::RotationRad;
use tangible_objects_model::Vec2;

use crate::LayerMask;
use crate::RoleClaim;

/// One command for the game host.
///
/// The tool never mutates session state itself - it wraps the command
/// into a [`crate::GameRequest`] whose oneshot carries the reply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    /// Deals a fresh game (`phase: Deal → InProgress` once
    /// [`Self::Begin`] arrives). `None` uses the loaded map's
    /// defaults (`GameConfig::for_map`); `Some` deals with the given
    /// setup - needed for deal-shaping fields like detective count,
    /// ticket stocks or `allow_undo`, which `SetConfig` cannot change
    /// post-deal.
    NewGame {
        /// Optional explicit setup; `None` = map defaults.
        config: Option<GameConfig>,
    },
    /// Adjusts reveal schedule and max rounds pre-game (Deal phase
    /// only).
    SetConfig {
        /// Reveal preset override; `None` keeps the current choice.
        reveal: Option<RevealPreset>,
        /// Game length override; `None` keeps the current value.
        max_rounds: Option<RoundNumber>,
    },
    /// Leaves the Deal phase - moves are accepted afterwards.
    Begin,
    /// Seats an agent under a role claim; issues a `PlayerToken`.
    Join {
        /// Which seat the agent claims.
        claim: RoleClaim,
    },
    /// The role-scoped state for a token.
    GameState {
        /// The player's token.
        token: PlayerToken,
    },
    /// Legal destinations for a token's role.
    LegalMoves {
        /// The player's token.
        token: PlayerToken,
    },
    /// A move for a token's role (the host enforces `actor = role`).
    Move {
        /// The player's token.
        token: PlayerToken,
        /// The requested move; `request.actor` is overridden with the
        /// token's role.
        request: MoveRequest,
    },
    /// Lint the loaded map dataset (quality criteria Q1-Q9).
    MapLint,
    /// A PNG screenshot of the app window. Handled by the app-side
    /// drain, not the game host - the session knows no window.
    Screenshot,
    /// Every drawn entity's mm + px position under the current view
    /// transform (visual debugging).
    SceneLayout,
    /// Pan/rotate/zoom the view (app-side view state, not persisted).
    SetView {
        /// Zoom factor; `None` keeps.
        zoom: Option<MapScale>,
        /// Display rotation; `None` keeps.
        rotate: Option<DisplayRotation>,
        /// Pan offset on the surface; `None` keeps.
        pan: Option<Vec2<Millimeters>>,
        /// Compass rose position on the map; `None` keeps.
        compass: Option<Vec2<Millimeters>>,
    },
    /// Reloads the city dataset from disk (dev iteration - no
    /// restart needed). App-side.
    ReloadMap,
    /// Sets a virtual anchor pose on the surface - overrides the
    /// "place the tower" setup for headless/table-less runs.
    /// App-side.
    SetAnchor {
        /// Anchor center on the surface.
        center: Vec2<Millimeters>,
        /// Anchor orientation; `None` keeps upright.
        rotation_rad: Option<RotationRad>,
    },
    /// Global render factors (station/landmark/metro badge sizes,
    /// ring width, label font). App-side.
    SetParams {
        /// Station disc radius factor; `None` keeps.
        station_scale: Option<ScaleFactor>,
        /// Landmark pin size factor; `None` keeps.
        landmark_scale: Option<ScaleFactor>,
        /// Metro line-badge size factor; `None` keeps.
        metro_scale: Option<ScaleFactor>,
        /// Transport-ring stroke width factor; `None` keeps.
        ring_width: Option<ScaleFactor>,
        /// Label font size factor; `None` keeps.
        label_scale: Option<ScaleFactor>,
    },
    /// Toggle scene layers (app-side render state).
    SetLayers {
        /// Per-layer flags; `None` fields keep current values.
        mask: LayerMask,
    },
}
