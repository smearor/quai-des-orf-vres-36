//! Geometric shell types in map mm: aliases pinning the generic
//! `smearor-commons-geometry` shells to millimeters.

use smearor_commons_geometry::Millimeters;

/// A parametric line `origin + t * direction` in map mm.
pub(crate) type Line = smearor_commons_geometry::Line<Millimeters>;

/// A closed ring of points in map mm: footprints, block outlines,
/// park blobs.
pub type Polygon = smearor_commons_geometry::Polygon<Millimeters>;

/// An open line of points in map mm: river courses, streets,
/// decorative chords, edge routes.
pub type Polyline = smearor_commons_geometry::Polyline<Millimeters>;
