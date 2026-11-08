//! `House` - one procedural building footprint inside a block.

use crate::map::Polygon;

use crate::map::blocks::Depth;
use crate::map::blocks::Height;
use crate::map::blocks::Units;
use crate::map::blocks::Variant;

/// A procedural building inside a block: its footprint polygon
/// plus the frontage class and texture variant a style draws it
/// with. Row houses are quads (front edge on the outline ring);
/// corner houses wrap a sharp block vertex with a rounded facade
/// and have more vertices.
#[derive(Debug, Clone, PartialEq)]
pub struct House {
    /// Footprint polygon in map mm (≥3 vertices, closed implicitly).
    pub footprint: Polygon,
    /// Frontage width in units (1-4 for row houses, 1-2 per side
    /// for corner houses) - drives the unit split lines and the
    /// per-width texture variants.
    pub units: Units,
    /// Texture variant within the unit-width class (0..4) - styles
    /// pick fills and hatches by `(units, variant)`.
    pub variant: Variant,
    /// How deep the house reaches into the block - rolled in
    /// 0.5 mm steps per house.
    pub depth: Depth,
    /// Extrusion factor for the isometric modes (`STANDARD` = the
    /// block's base height) - rolled in 0.25 steps, landmark
    /// volumes use `LANDMARK`.
    pub height: Height,
}
