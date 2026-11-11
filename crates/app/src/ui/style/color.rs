//! `RgbColor` - a normalized (0.0-1.0) RGB color for style palettes.

use smearor_commons_color::Color;

/// A normalized RGB color (each channel 0.0-1.0).
///
/// Alias for [`Color<f64>`]; conversion, luminance/blend math and the
/// cairo source helpers (`apply`/`apply_alpha`) are provided by
/// `smearor-commons-color`.
pub type RgbColor = Color<f64>;
