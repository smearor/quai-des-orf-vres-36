//! `CaptionStyle` - the place-label look shared by the style
//! renderers.

use crate::ui::style::LabelOrientation;
use smearor_commons_color::Color;
use smearor_commons_geometry::Millimeters;

/// Caption look for the place labels: glyph height, the halo
/// and ink colors, and the tabletop orientation.
pub struct CaptionStyle {
    /// Glyph height.
    pub font: Millimeters,
    /// The wide outline stroke lifting the text off its
    /// background.
    pub halo: Color<f64>,
    /// The glyph fill.
    pub ink: Color<f64>,
    /// Tabletop orientation (`Radial` faces the nearest edge,
    /// `Screen` stays upright).
    pub orientation: LabelOrientation,
}

impl CaptionStyle {
    /// The district caption look.
    pub fn district(halo: Color<f64>, ink: Color<f64>, orientation: LabelOrientation) -> Self {
        Self {
            font: Millimeters::new(3.4),
            halo,
            ink,
            orientation,
        }
    }

    /// The park caption look (smaller than the district name).
    pub fn park(halo: Color<f64>, ink: Color<f64>, orientation: LabelOrientation) -> Self {
        Self {
            font: Millimeters::new(2.4),
            halo,
            ink,
            orientation,
        }
    }
}
