//! `RgbColor` - a normalized (0.0-1.0) RGB color for style palettes.

use cairo::Context;
use smearor_commons_color::Color;
use std::ops::Deref;

use crate::ui::style::Alpha;
use crate::ui::style::Luminance;

/// A normalized RGB color (each channel 0.0-1.0).
///
/// Styles carry palette entries as `RgbColor`; `apply`/`apply_alpha`
/// set it on a cairo context - no raw channel triplets at call sites.
///
/// Thin newtype around [`Color<f64>`]; conversion, luminance/blend
/// math and the cairo source helpers are provided by
/// `smearor-commons-color`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct RgbColor(pub Color<f64>);

impl RgbColor {
    /// An opaque color.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Color::<f64>::new(r, g, b))
    }

    /// From 0-255 channels.
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self(Color::<u8>::new(r, g, b).into())
    }

    /// The perceived luminance (ITU-R BT.709 weights, 0-1).
    pub fn luminance(&self) -> Luminance {
        Luminance::new(self.0.luminance_709())
    }

    /// The color lifted to at least `min` luminance by blending
    /// toward white - dark line colors stay readable on a dark
    /// field (lerp toward white moves luminance linearly toward 1).
    pub fn with_min_luminance(&self, min: Luminance) -> Self {
        Self(self.0.with_min_luminance(min.as_f64()))
    }

    /// Linear blend toward `other` by `t` (0 = self, 1 = other).
    pub fn lerp(&self, other: &RgbColor, t: f64) -> Self {
        Self(self.0.lerp(&other.0, t))
    }

    /// Sets this color as the cairo source.
    pub fn apply(&self, ctx: &Context) {
        self.0.apply(ctx);
    }

    /// Sets this color as the cairo source, with alpha.
    pub fn apply_alpha(&self, ctx: &Context, alpha: Alpha) {
        self.0.apply_alpha(ctx, alpha.as_f64());
    }
}

impl Deref for RgbColor {
    type Target = Color<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Color<f64>> for RgbColor {
    fn from(color: Color<f64>) -> Self {
        Self(color)
    }
}

impl From<RgbColor> for Color<f64> {
    fn from(color: RgbColor) -> Self {
        color.0
    }
}
