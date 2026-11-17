//! `AnchorFrac` - a document-space anchor fraction for artwork
//! placement.

use smearor_commons_geometry::Fraction;
use smearor_commons_geometry::Vec2;

/// A point inside the SVG document, normalized to `0.0-1.0` in both
/// axes. The draw call places this fraction of the document on the
/// spec's map position - e.g. `PIN_TIP` puts a pin's tip on the
/// station coordinate.
pub type AnchorFrac = Vec2<Fraction>;

/// Document center.
pub const CENTER: AnchorFrac = Vec2::new(0.5, 0.5);

/// Bottom center - the pin tip of the station-marker masters
/// (~95% of the 60x110 viewBox).
pub const PIN_TIP: AnchorFrac = Vec2::new(0.5, 0.95);
