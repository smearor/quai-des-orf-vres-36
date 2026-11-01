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
    /// Pixels per surface millimeter.
    pub px_per_mm: PixelsPerMm,
}
