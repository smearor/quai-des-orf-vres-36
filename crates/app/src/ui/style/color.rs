//! `RgbColor` - a normalized (0.0-1.0) RGB color for style palettes.

use cairo::Context;

use crate::ui::style::Alpha;
use crate::ui::style::Luminance;

/// A normalized RGB color (each channel 0.0-1.0).
///
/// Styles carry palette entries as `RgbColor`; `apply`/`apply_alpha`
/// set it on a cairo context - no raw channel triplets at call sites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbColor {
    /// Red channel.
    pub r: f64,
    /// Green channel.
    pub g: f64,
    /// Blue channel.
    pub b: f64,
}

impl RgbColor {
    /// An opaque color.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// From 0-255 channels.
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self::new(f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0)
    }

    /// The perceived luminance (ITU-R BT.709 weights, 0-1).
    pub fn luminance(&self) -> Luminance {
        Luminance::new(self.r * 0.2126 + self.g * 0.7152 + self.b * 0.0722)
    }

    /// The color lifted to at least `min` luminance by blending
    /// toward white - dark line colors stay readable on a dark
    /// field (lerp toward white moves luminance linearly toward 1).
    pub fn with_min_luminance(&self, min: Luminance) -> Self {
        let lum = self.luminance().as_f64();
        if lum >= min.as_f64() {
            return *self;
        }
        let t = ((min.as_f64() - lum) / (1.0 - lum)).clamp(0.0, 1.0);
        Self::new(self.r + (1.0 - self.r) * t, self.g + (1.0 - self.g) * t, self.b + (1.0 - self.b) * t)
    }

    /// Sets this color as the cairo source.
    pub fn apply(&self, ctx: &Context) {
        ctx.set_source_rgb(self.r, self.g, self.b);
    }

    /// Sets this color as the cairo source, with alpha.
    pub fn apply_alpha(&self, ctx: &Context, alpha: Alpha) {
        ctx.set_source_rgba(self.r, self.g, self.b, alpha.as_f64());
    }
}
