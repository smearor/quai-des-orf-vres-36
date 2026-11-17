//! `DocumentSize` - an SVG document's intrinsic size in its own
//! (viewBox) units.

use smearor_commons_geometry::Size;

/// An SVG's intrinsic width/height in document units - either the
/// absolute `width`/`height` or the `viewBox` extent when the
/// document is percentage-sized (our generated assets are
/// `width="100%"`).
pub type DocumentSize = Size<f64>;
