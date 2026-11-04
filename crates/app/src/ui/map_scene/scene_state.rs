//! `SceneState` - everything the scene draws in one render pass.

use std::rc::Rc;

use qdo36_map::GameMap;
use qdo36_map::StationId;
use qdo36_tangibles::AnchorConfig;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::MapScale;
use qdo36_tangibles::SurfaceMm;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

use crate::ui::art::ArtAssets;
use crate::ui::map_scene::RenderParams;
use crate::ui::map_scene::SceneLayers;
use crate::ui::style::MapStyle;

/// Everything the scene draws - the map, anchor, surface and view
/// state. Passed in whole via [`super::MapScene::update`].
#[derive(Clone)]
pub struct SceneState {
    /// The loaded city map.
    pub map: Rc<GameMap>,
    /// Anchor calibration (anchor station + heading offset).
    pub config: AnchorConfig,
    /// Current anchor state (tracked or virtual pose).
    pub anchor: AnchorState,
    /// Physical surface size in mm (None: dev fallback px == mm).
    pub surface: Option<SurfaceMm>,
    /// View zoom.
    pub scale: MapScale,
    /// Highlighted station (tap selection).
    pub selected: Option<StationId>,
    /// Display rotation applied to surface-mm positions before the
    /// px scale (rotated desktop panels).
    pub display_rotation: DisplayRotation,
    /// The map look (comic / neon / belle epoque).
    pub style: MapStyle,
    /// The generated SVG artwork (landmark pins, anchor, compass).
    pub assets: Rc<ArtAssets>,
    /// Overlay hint shown while the anchor is virtual (tower lifted
    /// / not yet placed).
    pub anchor_hint: Option<String>,
    /// Which scene sections render (MCP `set_layers`).
    pub layers: SceneLayers,
    /// View pan offset in surface-mm (MCP `set_view`).
    pub pan: Vec2<Millimeters>,
    /// Compass rose position in map-mm (MCP `set_view compass_mm`).
    pub compass_pos: Vec2<Millimeters>,
    /// Global render factors (MCP `set_params` / UI controls).
    pub params: RenderParams,
}
