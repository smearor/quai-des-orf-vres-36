//! The Neon style palette (ARTWORK §2).

use crate::ui::style::RgbColor;
use crate::ui::style::StylePalette;

/// Near-black field, cyan glow family, pink selection.
pub(super) const PALETTE: StylePalette = StylePalette {
    // #05080D - near-black field.
    paper: RgbColor::new(0.02, 0.03, 0.05),
    // #1AF2FF - glow cyan (same as the accent; neon is monochrome-led).
    ink: RgbColor::new(0.10, 0.95, 1.0),
    // #1AF2FF - signature glow: edge cores, anchor, hint text.
    accent: RgbColor::new(0.10, 0.95, 1.0),
    // #4D7380 - dimmed cyan: virtual anchor.
    dim: RgbColor::new(0.30, 0.45, 0.50),
    // #1AF2FF - Seine glow (drawn at ~0.25 alpha).
    water: RgbColor::new(0.10, 0.95, 1.0),
    // #FF4DD9 - pink selection ring.
    select: RgbColor::new(1.0, 0.30, 0.85),
    // #1AF2FF - station discs glow on the dark field.
    station: RgbColor::new(0.10, 0.95, 1.0),
    // #66D9F2 - label text, slightly muted cyan.
    label: RgbColor::new(0.40, 0.85, 0.95),
    // #00050A - hint badge fill (drawn at ~0.85 alpha).
    panel: RgbColor::new(0.0, 0.02, 0.04),
    // #FFD91A - taxi yellow glow.
    taxi: RgbColor::new(1.0, 0.85, 0.10),
    // #33F266 - bus green glow.
    bus: RgbColor::new(0.20, 0.95, 0.40),
    // #00D9F2 - batobus cyan.
    batobus: RgbColor::new(0.0, 0.85, 0.95),
    // #FF7326 - underground secret routes, orange glow.
    underground: RgbColor::new(1.0, 0.45, 0.15),
};
