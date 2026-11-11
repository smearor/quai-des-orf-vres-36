//! The Belle Époque style palette (ARTWORK §2).

use crate::ui::style::StyleColor;
use crate::ui::style::StylePalette;
use smearor_commons_color::Color;
use smearor_commons_color::palette;
use std::sync::LazyLock;

/// Parchment field, sepia engraving ink, muted gold accents - the
/// same voice as the generated masters.
pub(crate) static PALETTE: LazyLock<StylePalette> = LazyLock::new(|| {
    palette! {
        // #F7EBD0 - parchment field.
        StyleColor::Paper => Color::<f64>::new(0.97, 0.92, 0.81),
        // #542C15 - sepia engraving ink: outlines, hairlines, anchor disc.
        StyleColor::Ink => Color::<f64>::new(0.33, 0.17, 0.08),
        // #B8871C - muted gold: ornamental rings, anchor ring, hint frame.
        StyleColor::Accent => Color::<f64>::new(0.72, 0.53, 0.11),
        // #8C734C - faded sepia for the virtual anchor.
        StyleColor::Dim => Color::<f64>::new(0.55, 0.45, 0.30),
        // #9EB8C7 - pale engraved Seine wash.
        StyleColor::Water => Color::<f64>::new(0.62, 0.72, 0.78),
        // #94ADBB - river body, a touch deeper than the bank wash.
        StyleColor::WaterFill => Color::<f64>::new(0.58, 0.68, 0.73),
        // #B8871C - gold selection ring (the same accent tone).
        StyleColor::Select => Color::<f64>::new(0.72, 0.53, 0.11),
        // #FAF2DB - station disc fill, lighter than the paper.
        StyleColor::Station => Color::<f64>::new(0.98, 0.95, 0.86),
        // #542C15 - sepia labels and hint text.
        StyleColor::Label => Color::<f64>::new(0.33, 0.17, 0.08),
        // #F7EBD0 - parchment hint badge (drawn at ~0.94 alpha).
        StyleColor::Panel => Color::<f64>::new(0.97, 0.92, 0.81),
        // #C79914 - taxi, muted ochre.
        StyleColor::Taxi => Color::<f64>::new(0.78, 0.60, 0.08),
        // #386B48 - bus, period green.
        StyleColor::Bus => Color::<f64>::new(0.22, 0.42, 0.28),
        // #2E7385 - batobus, faded teal.
        StyleColor::Batobus => Color::<f64>::new(0.18, 0.45, 0.52),
        // #542C15 - underground secret routes, sepia dashes.
        StyleColor::Underground => Color::<f64>::new(0.33, 0.17, 0.08),
        // #6E6252 - neutral edge ink in sepia (standard view).
        StyleColor::Edge => Color::<f64>::new(0.43, 0.38, 0.32),
        // #6B4A8E - period mauve: generic metro ring mark and the
        // metro lane in parallel-transports view.
        StyleColor::Metro => Color::<f64>::new(0.42, 0.29, 0.56),
        // #C9B98F - faded sepia: city-block outlines on the
        // parchment field.
        StyleColor::Block => Color::<f64>::new(0.79, 0.73, 0.56),
        // #E4D3A9 - aged-stone fill for the procedural houses.
        StyleColor::Building => Color::<f64>::new(0.89, 0.83, 0.66),
        StyleColor::Park => Color::<f64>::new(0.62, 0.72, 0.50),
        // #A98F61 - engraved sepia roof lines, darker than the
        // building fill.
        StyleColor::Roof => Color::<f64>::new(0.66, 0.56, 0.38),
        // #8F7D5C - quiet sepia for decorative streets.
        StyleColor::Street => Color::<f64>::new(0.56, 0.49, 0.36),
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
