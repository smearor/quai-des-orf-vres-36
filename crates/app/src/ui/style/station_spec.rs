//! `StationSpec` - everything a style needs to draw one station.

use crate::ui::style::PxPos;
use qdo36_map::GameMap;
use qdo36_map::ImportanceClass;
use qdo36_map::LineName;
use qdo36_map::Transport;
use qdo36_tangibles::ScaleFactor;
use smearor_commons_geometry::PixelsPerMm;

/// A station node under the current view: base disc, one ring per
/// served transport and - inside the metro overlay - a pie-split
/// fill in the colors of the lines serving it.
#[derive(Debug)]
pub struct StationSpec<'a> {
    /// Station center in widget px.
    pub pos: PxPos,
    /// Transports serving the station (drives the rings).
    pub served: &'a [Transport],
    /// Metro lines serving the station (drives the pie fill).
    pub lines: &'a [LineName],
    /// The map - line-color lookup for the pie fill.
    pub map: &'a GameMap,
    /// Metro overlay active: the disc fills in line colors, split
    /// into `lines.len()` sectors when several lines share it.
    pub metro_overlay: bool,
    /// Strategic importance class - scales the disc radius
    /// (`Minor` smaller, `Typical` base, `Hub` larger).
    pub importance: Option<ImportanceClass>,
    /// Selection ring drawn when true.
    pub selected: bool,
    /// Global disc radius factor (`RenderParams.station_scale`).
    pub station_scale: ScaleFactor,
    /// Global transport-ring width factor (`RenderParams.ring_width`).
    pub ring_width: ScaleFactor,
    /// Pixels per surface millimeter.
    pub px_per_mm: PixelsPerMm,
}
