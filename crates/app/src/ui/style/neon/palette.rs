//! The Neon style palette (ARTWORK §2).

use crate::ui::style::StyleColor;
use crate::ui::style::StylePalette;
use smearor_commons_color::Color;
use smearor_commons_color::palette;
use std::sync::LazyLock;

/// Near-black field, cyan glow family, pink selection.
pub(crate) static PALETTE: LazyLock<StylePalette> = LazyLock::new(|| {
    palette! {
        // #05080D - near-black field.
        StyleColor::Paper => Color::<f64>::new(0.02, 0.03, 0.05),
        // #1AF2FF - glow cyan (same as the accent; neon is monochrome-led).
        StyleColor::Ink => Color::<f64>::new(0.10, 0.95, 1.0),
        // #1AF2FF - signature glow: edge cores, anchor, hint text.
        StyleColor::Accent => Color::<f64>::new(0.10, 0.95, 1.0),
        // #4D7380 - dimmed cyan: virtual anchor.
        StyleColor::Dim => Color::<f64>::new(0.30, 0.45, 0.50),
        // #1AF2FF - glowing Seine bank lines.
        StyleColor::Water => Color::<f64>::new(0.10, 0.95, 1.0),
        // #0A2130 - dark river body, distinct from the near-black field.
        StyleColor::WaterFill => Color::<f64>::new(0.04, 0.13, 0.19),
        // #FF4DD9 - pink selection ring.
        StyleColor::Select => Color::<f64>::new(1.0, 0.30, 0.85),
        // #1AF2FF - station discs glow on the dark field.
        StyleColor::Station => Color::<f64>::new(0.10, 0.95, 1.0),
        // #66D9F2 - label text, slightly muted cyan.
        StyleColor::Label => Color::<f64>::new(0.40, 0.85, 0.95),
        // #00050A - hint badge fill (drawn at ~0.85 alpha).
        StyleColor::Panel => Color::<f64>::new(0.0, 0.02, 0.04),
        // #FFD91A - taxi yellow glow.
        StyleColor::Taxi => Color::<f64>::new(1.0, 0.85, 0.10),
        // #33F266 - bus green glow.
        StyleColor::Bus => Color::<f64>::new(0.20, 0.95, 0.40),
        // #00D9F2 - batobus cyan.
        StyleColor::Batobus => Color::<f64>::new(0.0, 0.85, 0.95),
        // #FF7326 - underground secret routes, orange glow.
        StyleColor::Underground => Color::<f64>::new(1.0, 0.45, 0.15),
        // #1A2B38 - dim neutral edge ink (standard view, no highlight).
        StyleColor::Edge => Color::<f64>::new(0.10, 0.17, 0.22),
        // #B44DFF - purple glow: generic metro ring mark and the
        // metro lane in parallel-transports view.
        StyleColor::Metro => Color::<f64>::new(0.71, 0.30, 1.0),
        // #14505E - low-glow cyan: city-block outlines, dimmer than
        // the route lanes.
        StyleColor::Block => Color::<f64>::new(0.08, 0.31, 0.37),
        // #0C2733 - deep teal fill for the procedural houses.
        StyleColor::Building => Color::<f64>::new(0.05, 0.15, 0.20),
        StyleColor::Park => Color::<f64>::new(0.10, 0.38, 0.26),
        // #2E7A8C - lifted cyan roof lines on the dark building fill.
        StyleColor::Roof => Color::<f64>::new(0.18, 0.48, 0.55),
        // #3A4A52 - muted slate, distinct from block cyan and the
        // orange underground routes.
        StyleColor::Street => Color::<f64>::new(0.23, 0.29, 0.32),
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
