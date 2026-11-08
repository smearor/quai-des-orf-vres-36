//! `SceneState` - everything the scene draws in one render pass.

use std::cell::OnceCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use qdo36_map::BlockLayout;
use qdo36_map::BridgeDeck;
use qdo36_map::GameMap;
use qdo36_map::Polyline;
use qdo36_map::Segment;
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
use crate::ui::map_scene::blocks::BlocksGeometry;
use crate::ui::style::MapStyle;
use crate::ui::style::WATER_HALF_WIDTH;

/// Everything the scene draws - the map, anchor, surface and view
/// state. Passed in whole via [`crate::ui::map_scene::MapScene::update`].
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
    /// The map's city-block division - computed once on first use;
    /// the map is immutable so the layout never goes stale.
    pub block_layout: OnceCell<Rc<BlockLayout>>,
    /// The rendered block geometry (inset outlines + houses +
    /// ground masks) for the current view mode - recomputed when
    /// the mode key changes, since the overlays widen the street
    /// corridors.
    pub block_shapes: RefCell<Option<(u8, Rc<BlocksGeometry>)>>,
    /// The bridge-station decks - computed once on first use; the
    /// map is immutable so they never go stale.
    pub decks: OnceCell<HashMap<StationId, BridgeDeck>>,
    /// The smoothed water courses - computed once on first use;
    /// the map is immutable so they never go stale.
    pub water_courses: OnceCell<Vec<Polyline>>,
    /// The water courses' segments for bridge crossing tests -
    /// computed once on first use.
    pub water_segments: OnceCell<Vec<Segment<Millimeters>>>,
}

impl SceneState {
    /// The map's city-block division (lazily computed).
    pub fn block_layout(&self) -> &Rc<BlockLayout> {
        self.block_layout.get_or_init(|| Rc::new(self.map.block_layout(WATER_HALF_WIDTH)))
    }

    /// Bridge-station decks: one per `bridge` station, perpendicular
    /// to the smoothed course, station marker mid-deck. Empty while
    /// the water layer is off - an invisible deck must not reroute
    /// the roads either (lazily computed).
    pub fn decks(&self) -> &HashMap<StationId, BridgeDeck> {
        /// No decks while the water layer is off.
        static EMPTY: LazyLock<HashMap<StationId, BridgeDeck>> = LazyLock::new(HashMap::new);
        if self.layers.water {
            self.decks.get_or_init(|| self.map.bridge_decks(WATER_HALF_WIDTH))
        } else {
            &EMPTY
        }
    }

    /// The Chaikin-smoothed water courses - the same courses feed
    /// the river bands, the Batobus routes and the bridge decks
    /// (lazily computed).
    pub fn water_courses(&self) -> &[Polyline] {
        self.water_courses.get_or_init(|| self.map.smoothed_water())
    }

    /// The water courses' segments - the bridge crossing tests
    /// run on these (lazily computed).
    pub fn water_segments(&self) -> &[Segment<Millimeters>] {
        self.water_segments
            .get_or_init(|| self.water_courses().iter().flat_map(|course| course.segments()).collect())
    }
}
