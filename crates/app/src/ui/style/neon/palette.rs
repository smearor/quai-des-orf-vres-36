//! The Neon style palette (ARTWORK §2).

use crate::ui::style::RgbColor;
use crate::ui::style::StylePalette;

/// Near-black field, cyan glow family, pink selection.
pub(crate) const PALETTE: StylePalette = StylePalette {
    // #05080D - near-black field.
    paper: RgbColor::new(0.02, 0.03, 0.05),
    // #1AF2FF - glow cyan (same as the accent; neon is monochrome-led).
    ink: RgbColor::new(0.10, 0.95, 1.0),
    // #1AF2FF - signature glow: edge cores, anchor, hint text.
    accent: RgbColor::new(0.10, 0.95, 1.0),
    // #4D7380 - dimmed cyan: virtual anchor.
    dim: RgbColor::new(0.30, 0.45, 0.50),
    // #1AF2FF - glowing Seine bank lines.
    water: RgbColor::new(0.10, 0.95, 1.0),
    // #0A2130 - dark river body, distinct from the near-black field.
    water_fill: RgbColor::new(0.04, 0.13, 0.19),
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
    // #1A2B38 - dim neutral edge ink (standard view, no highlight).
    edge: RgbColor::new(0.10, 0.17, 0.22),
    // #B44DFF - purple glow: generic metro ring mark and the
    // metro lane in parallel-transports view.
    metro: RgbColor::new(0.71, 0.30, 1.0),
    // #14505E - low-glow cyan: city-block outlines, dimmer than
    // the route lanes.
    block: RgbColor::new(0.08, 0.31, 0.37),
    // #0C2733 - deep teal fill for the procedural houses.
    building: RgbColor::new(0.05, 0.15, 0.20),
    park: RgbColor::new(0.10, 0.38, 0.26),
    // #2E7A8C - lifted cyan roof lines on the dark building fill.
    roof: RgbColor::new(0.18, 0.48, 0.55),
    // #3A4A52 - muted slate, distinct from block cyan and the
    // orange underground routes.
    street: RgbColor::new(0.23, 0.29, 0.32),
};
