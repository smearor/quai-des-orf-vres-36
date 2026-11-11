//! The Comic style palette (ARTWORK §2).

use crate::ui::style::StyleColor;
use crate::ui::style::StylePalette;
use smearor_commons_color::Color;
use smearor_commons_color::palette;
use std::sync::LazyLock;

/// Flat fills on a dark comic-board field, cream station discs,
/// orange accents.
pub(crate) static PALETTE: LazyLock<StylePalette> = LazyLock::new(|| {
    palette! {
        // #1A1714 - dark field the board paints on.
        StyleColor::Paper => Color::<f64>::new(0.10, 0.09, 0.08),
        // #26211C - node outlines.
        StyleColor::Ink => Color::<f64>::new(0.15, 0.13, 0.11),
        // #F27333 - anchor disc when the tower is tracked.
        StyleColor::Accent => Color::<f64>::new(0.95, 0.45, 0.20),
        // #8C8073 - virtual anchor, landmark rings.
        StyleColor::Dim => Color::<f64>::new(0.55, 0.50, 0.45),
        // #2E526B - halftone Seine bank lines.
        StyleColor::Water => Color::<f64>::new(0.18, 0.32, 0.42),
        // #0F1C29 - dark river body, cooler than the warm field.
        StyleColor::WaterFill => Color::<f64>::new(0.06, 0.11, 0.16),
        // #F28C26 - selection ring.
        StyleColor::Select => Color::<f64>::new(0.95, 0.55, 0.15),
        // #EBE0CC - cream station discs.
        StyleColor::Station => Color::<f64>::new(0.92, 0.88, 0.80),
        // #E0D9C7 - station labels and hint text.
        StyleColor::Label => Color::<f64>::new(0.88, 0.85, 0.78),
        // #000000 - hint badge fill (drawn at ~0.7 alpha).
        StyleColor::Panel => Color::<f64>::new(0.0, 0.0, 0.0),
        // #F5C518 - taxi yellow.
        StyleColor::Taxi => Color::<f64>::new(0.96, 0.77, 0.09),
        // #2E8C56 - bus green.
        StyleColor::Bus => Color::<f64>::new(0.18, 0.55, 0.34),
        // #3399DC - batobus blue.
        StyleColor::Batobus => Color::<f64>::new(0.20, 0.60, 0.86),
        // #6B5238 - underground secret routes, dashed.
        StyleColor::Underground => Color::<f64>::new(0.42, 0.32, 0.22),
        // #4A443C - neutral edge ink (standard view, no highlight).
        StyleColor::Edge => Color::<f64>::new(0.29, 0.27, 0.24),
        // #7C3AED - bold purple: generic metro ring mark and the
        // metro lane in parallel-transports view.
        StyleColor::Metro => Color::<f64>::new(0.49, 0.23, 0.93),
        // #3D3229 - quiet warm brown: city-block outlines over the
        // dark field.
        StyleColor::Block => Color::<f64>::new(0.24, 0.20, 0.16),
        // #4A3B28 - warm brick fill for the procedural houses.
        StyleColor::Building => Color::<f64>::new(0.29, 0.23, 0.16),
        StyleColor::Park => Color::<f64>::new(0.48, 0.58, 0.30),
        // #9E8B64 - light sepia roof lines on the dark building fill.
        StyleColor::Roof => Color::<f64>::new(0.62, 0.55, 0.39),
        // #6B5F4C - quiet warm grey for decorative streets.
        StyleColor::Street => Color::<f64>::new(0.42, 0.37, 0.30),
    }
    .into()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_fills_every_style_color_slot() {
        assert!(PALETTE.is_complete(), "unfilled slots: {:?}", PALETTE.missing());
    }
}
