//! `AnchorFrac` - a document-space anchor fraction for artwork
//! placement.

/// A point inside the SVG document, normalized to `0.0-1.0` in both
/// axes. The draw call places this fraction of the document on the
/// spec's map position - e.g. `PIN_TIP` puts a pin's tip on the
/// station coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchorFrac {
    /// Horizontal fraction: `0.0` = document left edge.
    pub x: f64,
    /// Vertical fraction: `0.0` = document top edge.
    pub y: f64,
}

impl AnchorFrac {
    /// Document center.
    pub const CENTER: Self = Self::new(0.5, 0.5);
    /// Bottom center - the pin tip of the station-marker masters
    /// (~95% of the 60x110 viewBox).
    pub const PIN_TIP: Self = Self::new(0.5, 0.95);

    /// An anchor fraction.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
