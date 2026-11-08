//! `CaptionStyle` - the place-label look shared by the style
//! renderers.

use tangible_objects_model::Millimeters;

use crate::ui::style::LabelOrientation;
use crate::ui::style::RgbColor;

/// Caption look for the place labels: glyph height, the halo
/// and ink colors, and the tabletop orientation.
pub struct CaptionStyle {
    /// Glyph height.
    pub font: Millimeters,
    /// The wide outline stroke lifting the text off its
    /// background.
    pub halo: RgbColor,
    /// The glyph fill.
    pub ink: RgbColor,
    /// Tabletop orientation (`Radial` faces the nearest edge,
    /// `Screen` stays upright).
    pub orientation: LabelOrientation,
}

impl CaptionStyle {
    /// The district caption look.
    pub fn district(halo: RgbColor, ink: RgbColor, orientation: LabelOrientation) -> Self {
        Self {
            font: Millimeters::new(3.4),
            halo,
            ink,
            orientation,
        }
    }

    /// The park caption look (smaller than the district name).
    pub fn park(halo: RgbColor, ink: RgbColor, orientation: LabelOrientation) -> Self {
        Self {
            font: Millimeters::new(2.4),
            halo,
            ink,
            orientation,
        }
    }
}
