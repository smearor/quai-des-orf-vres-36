//! `DocumentSize` - an SVG document's intrinsic size in its own
//! (viewBox) units.

/// An SVG's intrinsic width/height in document units - either the
/// absolute `width`/`height` or the `viewBox` extent when the
/// document is percentage-sized (our generated assets are
/// `width="100%"`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DocumentSize {
    /// Document width.
    pub width: f64,
    /// Document height.
    pub height: f64,
}

impl DocumentSize {
    /// A document size.
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// The aspect ratio (width / height), `None` when degenerate.
    pub fn aspect(&self) -> Option<f64> {
        (self.width > 0.0 && self.height > 0.0).then_some(self.width / self.height)
    }
}
