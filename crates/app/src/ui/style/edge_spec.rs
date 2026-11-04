//! `EdgeSpec` - one rendered edge as the `StyleRenderer` sees it.

use qdo36_map::GameMap;
use qdo36_map::LineName;
use qdo36_map::Transport;
use tangible_objects_model::PixelsPerMm;

use crate::ui::style::PxPos;

/// One rendered edge - endpoints already in widget px, plus the
/// domain data the style may color by.
pub struct EdgeSpec<'a> {
    /// From-station position (px).
    pub from: PxPos,
    /// To-station position (px).
    pub to: PxPos,
    /// The edge's transport mode.
    pub transport: Transport,
    /// Transit line (metro only; colored per `[map.lines]`).
    pub line: Option<&'a LineName>,
    /// The map - line-color lookup.
    pub map: &'a GameMap,
    /// The drawn polyline in px (routed through intermediate stations);
    /// empty means a straight `from` -> `to` segment.
    pub route: Vec<PxPos>,
    /// Perpendicular px offset per drawn segment - parallel metro
    /// lines shift sideways so every line keeps its own color; one
    /// entry per drawn segment, empty means centered.
    pub offsets: Vec<f64>,
    /// Whether the edge draws in its transport/line color; `false`
    /// draws the neutral `edge` ink (standard view).
    pub highlight: bool,
    /// Pixels per surface millimeter.
    pub px_per_mm: PixelsPerMm,
}
