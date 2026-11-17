//! `BlockShapes` - one block's render geometry, cached per view mode.

use qdo36_map::House;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Polygon;
use smearor_commons_geometry::Polyline;

use crate::ui::style::DistrictLabel;
use crate::ui::style::ParkSpot;

/// A block's render geometry: the inset outline plus the
/// procedural house footprints inside - cached per view mode.
pub struct BlockShapes {
    /// Inset outline in map mm.
    pub outline: Polygon<Millimeters>,
    /// House footprints in map mm.
    pub buildings: Vec<House>,
    /// Decorative street chords crossing the block (visual only).
    pub streets: Vec<Polyline<Millimeters>>,
    /// Park blobs inside the block.
    pub parks: Vec<ParkSpot>,
    /// The district name the block carries, `None` when the block
    /// is too small to anchor a label.
    pub label: Option<DistrictLabel>,
}
