//! `RoutePlan` - an edge's drawn route plus its lane offsets.

use crate::ui::style::PxPos;
use qdo36_map::Polyline;
use tangible_objects_model::Pixels;

/// An edge's drawn polyline in px, its per-hop lane offsets and
/// the same polyline in map mm (for region tests).
pub struct RoutePlan {
    /// The drawn route polyline in widget px.
    pub route: Vec<PxPos>,
    /// Per-hop lane offsets in px; empty when no lane sharing
    /// applies.
    pub offsets: Vec<Pixels>,
    /// The same route in map mm - the region tests run in map
    /// space.
    pub route_mm: Polyline,
}
