//! The map scene widget: renders the GameMap under `MapTransform`
//! (map-mm -> surface-mm -> widget-px) on every snapshot.

use std::cell::RefCell;
use std::rc::Rc;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::prelude::WidgetExt;
use qdo36_map::GameMap;
use qdo36_map::LineName;
use qdo36_map::MapMeta;
use qdo36_map::Station;
use qdo36_map::Transport;
use qdo36_tangibles::AnchorConfig;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::MapScale;
use qdo36_tangibles::MapTransform;
use qdo36_tangibles::SurfaceMm;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Everything the scene draws - the map, anchor, surface and view
/// state. Passed in whole via [`MapScene::update`].
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
    pub selected: Option<qdo36_map::StationId>,
}

mod imp {
    use super::*;
    use gtk4::subclass::prelude::*;

    /// State held by the widget.
    pub struct MapSceneImp {
        /// The current scene state.
        pub state: RefCell<Option<Rc<SceneState>>>,
    }

    impl Default for MapSceneImp {
        fn default() -> Self {
            Self { state: RefCell::new(None) }
        }
    }

    #[glib::object_subclass]
    impl glib::subclass::types::ObjectSubclass for MapSceneImp {
        const NAME: &'static str = "Qdo36MapScene";
        type Type = super::MapScene;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for MapSceneImp {}

    impl WidgetImpl for MapSceneImp {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let widget = self.obj();
            let Some(state) = self.state.borrow().clone() else {
                return;
            };
            let width = widget.width() as f64;
            let height = widget.height() as f64;
            if width <= 0.0 || height <= 0.0 {
                return;
            }
            let rect = gtk4::graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
            let ctx = gtk4::prelude::SnapshotExt::append_cairo(snapshot, &rect);
            super::render(&ctx, &state, width, height);
        }
    }
}

glib::wrapper! {
    /// The map scene widget.
    pub struct MapScene(ObjectSubclass<imp::MapSceneImp>)
        @extends gtk4::Widget,
        @implements gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Accessible;
}

impl Default for MapScene {
    fn default() -> Self {
        Self::new()
    }
}

impl MapScene {
    /// Creates the scene widget.
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Replaces the scene state and queues a redraw.
    pub fn update(&self, state: Rc<SceneState>) {
        *self.imp().state.borrow_mut() = Some(state);
        self.queue_draw();
    }
}

/// Full scene render: background, backdrop, edges, stations, labels,
/// anchor marker.
fn render(ctx: &cairo::Context, state: &SceneState, width: f64, height: f64) {
    let map = state.map.as_ref();
    let bounds = map.bounds();

    // The transform maps map-mm -> surface-mm; the second stage maps
    // surface-mm -> widget px (letterboxed to keep aspect).
    let surface = state.surface.unwrap_or_else(|| SurfaceMm {
        width: Millimeters::new(width.max(bounds.width().as_f64())),
        height: Millimeters::new(height.max(bounds.height().as_f64())),
    });
    let px_per_mm = (width / surface.width.as_f64()).min(height / surface.height.as_f64());
    let offset_x = (width - surface.width.as_f64() * px_per_mm) / 2.0;
    let offset_y = (height - surface.height.as_f64() * px_per_mm) / 2.0;

    let transform = MapTransform::new(state.anchor.pose(), &state.config, state.scale);
    let to_px = |pos: Vec2<Millimeters>| {
        let surface_pos = transform.map_to_surface(pos);
        (offset_x + surface_pos.x * px_per_mm, offset_y + surface_pos.y * px_per_mm)
    };

    // Paper.
    ctx.set_source_rgb(0.10, 0.09, 0.08);
    ctx.paint().ok();

    draw_backdrop(ctx, map, &to_px, px_per_mm);
    draw_edges(ctx, map, &to_px, px_per_mm);
    draw_stations(ctx, map, &to_px, px_per_mm, state.selected);
    draw_anchor(ctx, state.anchor, &to_px, px_per_mm, &state.config);
}

/// River polylines + landmark dots - gameplay-inert dressing.
fn draw_backdrop(ctx: &cairo::Context, map: &GameMap, to_px: &impl Fn(Vec2<Millimeters>) -> (f64, f64), px_per_mm: f64) {
    let backdrop = map.backdrop();
    ctx.set_line_width(6.0 * px_per_mm);
    ctx.set_line_cap(cairo::LineCap::Round);
    ctx.set_source_rgb(0.18, 0.32, 0.42);
    for polyline in &backdrop.water {
        let mut first = true;
        ctx.new_path();
        for point in polyline.points() {
            let (x, y) = to_px(*point);
            if first {
                ctx.move_to(x, y);
                first = false;
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.stroke().ok();
    }
    // Landmark dots: a small ring keeps them below the station graph.
    ctx.set_source_rgb(0.45, 0.42, 0.38);
    ctx.set_line_width(1.5);
    for landmark in &backdrop.landmarks {
        let (x, y) = to_px(landmark.pos);
        ctx.arc(x, y, 2.2 * px_per_mm.max(0.5), 0.0, std::f64::consts::TAU);
        ctx.stroke().ok();
    }
}

/// Edges, colored per transport (metro uses the declared line color).
fn draw_edges(ctx: &cairo::Context, map: &GameMap, to_px: &impl Fn(Vec2<Millimeters>) -> (f64, f64), px_per_mm: f64) {
    ctx.set_line_cap(cairo::LineCap::Round);
    for edge in map.edges() {
        let Some(from) = map.station(edge.from) else { continue };
        let Some(to) = map.station(edge.to) else { continue };
        let (x1, y1) = to_px(from.pos);
        let (x2, y2) = to_px(to.pos);
        match edge.transport {
            Transport::Underground => {
                ctx.set_dash(&[4.0 * px_per_mm, 3.0 * px_per_mm], 0.0);
                ctx.set_source_rgb(0.42, 0.32, 0.22);
            }
            _ => {
                ctx.set_dash(&[], 0.0);
                let (r, g, b) = transport_color(edge.transport, edge.line.as_ref(), map.meta());
                ctx.set_source_rgb(r, g, b);
            }
        }
        ctx.set_line_width(2.4 * px_per_mm.max(0.5));
        ctx.new_path();
        ctx.move_to(x1, y1);
        ctx.line_to(x2, y2);
        ctx.stroke().ok();
    }
    ctx.set_dash(&[], 0.0);
}

/// Station nodes + name labels; selection ring on `selected`.
fn draw_stations(
    ctx: &cairo::Context,
    map: &GameMap,
    to_px: &impl Fn(Vec2<Millimeters>) -> (f64, f64),
    px_per_mm: f64,
    selected: Option<qdo36_map::StationId>,
) {
    let radius = 3.0 * px_per_mm.max(0.4);
    for station in map.stations() {
        let (x, y) = to_px(station.pos);
        // Node: light fill, dark outline.
        ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU);
        ctx.set_source_rgb(0.92, 0.88, 0.80);
        ctx.fill_preserve().ok();
        ctx.set_line_width(1.4);
        ctx.set_source_rgb(0.15, 0.13, 0.11);
        ctx.stroke().ok();
        if selected == Some(station.id) {
            ctx.arc(x, y, radius + 3.0 * px_per_mm.max(0.4), 0.0, std::f64::consts::TAU);
            ctx.set_source_rgb(0.95, 0.55, 0.15);
            ctx.set_line_width(2.5);
            ctx.stroke().ok();
        }
        draw_label(ctx, station, x, y, radius, px_per_mm);
    }
}

/// Station name below the node.
fn draw_label(ctx: &cairo::Context, station: &Station, x: f64, y: f64, radius: f64, px_per_mm: f64) {
    ctx.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    ctx.set_font_size((4.0 * px_per_mm).max(8.0));
    let name = station.name.as_str();
    let extents = ctx.text_extents(name).unwrap_or_else(|_| cairo::TextExtents::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
    ctx.move_to(x - extents.width() / 2.0, y + radius + (5.5 * px_per_mm).max(10.0));
    ctx.set_source_rgb(0.88, 0.85, 0.78);
    ctx.show_text(name).ok();
}

/// The anchor marker: a ring at the anchor's surface pose, filled
/// when tracked, hollow while virtual (tower lifted).
fn draw_anchor(ctx: &cairo::Context, anchor: AnchorState, to_px: &impl Fn(Vec2<Millimeters>) -> (f64, f64), px_per_mm: f64, config: &AnchorConfig) {
    // The anchor pos in map-mm projects onto the tangible centroid.
    let (x, y) = to_px(config.anchor_map_pos);
    let radius = 6.0 * px_per_mm.max(0.5);
    let (r, g, b) = match anchor {
        AnchorState::Tracked(_) => (0.95, 0.45, 0.20),
        AnchorState::Virtual(_) => (0.55, 0.50, 0.45),
    };
    ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU);
    match anchor {
        AnchorState::Tracked(_) => {
            ctx.set_source_rgb(r, g, b);
            ctx.fill_preserve().ok();
            ctx.set_source_rgb(1.0, 1.0, 1.0);
        }
        AnchorState::Virtual(_) => ctx.set_source_rgb(r, g, b),
    }
    ctx.set_line_width(1.8);
    ctx.stroke().ok();
}

/// Edge color per transport; metro edges carry their declared line
/// color, batobus/underground get fixed secret-route colors.
fn transport_color(transport: Transport, line: Option<&LineName>, meta: &MapMeta) -> (f64, f64, f64) {
    match transport {
        Transport::Taxi => (0.96, 0.77, 0.09),        // SY taxi yellow
        Transport::Bus => (0.18, 0.55, 0.34),         // SY bus green
        Transport::Batobus => (0.20, 0.60, 0.86),     // river cyan
        Transport::Underground => (0.42, 0.32, 0.22), // tunnel brown
        Transport::Metro => line
            .and_then(|name| meta.lines.get(name))
            .map(|color| {
                let [r, g, b] = color.rgb_u8();
                (f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0)
            })
            .unwrap_or((0.75, 0.22, 0.17)), // SY metro red fallback
    }
}
