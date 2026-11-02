//! The Belle Époque style palette (ARTWORK §2).

use crate::ui::style::RgbColor;
use crate::ui::style::StylePalette;

/// Parchment field, sepia engraving ink, muted gold accents - the
/// same voice as the generated masters.
pub(super) const PALETTE: StylePalette = StylePalette {
    // #F7EBD0 - parchment field.
    paper: RgbColor::new(0.97, 0.92, 0.81),
    // #542C15 - sepia engraving ink: outlines, hairlines, anchor disc.
    ink: RgbColor::new(0.33, 0.17, 0.08),
    // #B8871C - muted gold: ornamental rings, anchor ring, hint frame.
    accent: RgbColor::new(0.72, 0.53, 0.11),
    // #8C734C - faded sepia for the virtual anchor.
    dim: RgbColor::new(0.55, 0.45, 0.30),
    // #9EB8C7 - pale engraved Seine wash.
    water: RgbColor::new(0.62, 0.72, 0.78),
    // #B8871C - gold selection ring (the same accent tone).
    select: RgbColor::new(0.72, 0.53, 0.11),
    // #FAF2DB - station disc fill, lighter than the paper.
    station: RgbColor::new(0.98, 0.95, 0.86),
    // #542C15 - sepia labels and hint text.
    label: RgbColor::new(0.33, 0.17, 0.08),
    // #F7EBD0 - parchment hint badge (drawn at ~0.94 alpha).
    panel: RgbColor::new(0.97, 0.92, 0.81),
    // #C79914 - taxi, muted ochre.
    taxi: RgbColor::new(0.78, 0.60, 0.08),
    // #386B48 - bus, period green.
    bus: RgbColor::new(0.22, 0.42, 0.28),
    // #2E7385 - batobus, faded teal.
    batobus: RgbColor::new(0.18, 0.45, 0.52),
    // #542C15 - underground secret routes, sepia dashes.
    underground: RgbColor::new(0.33, 0.17, 0.08),
};
