//! The Comic style palette (ARTWORK §2).

use crate::ui::style::RgbColor;
use crate::ui::style::StylePalette;

/// Flat fills on a dark comic-board field, cream station discs,
/// orange accents.
pub(super) const PALETTE: StylePalette = StylePalette {
    // #1A1714 - dark field the board paints on.
    paper: RgbColor::new(0.10, 0.09, 0.08),
    // #26211C - node outlines.
    ink: RgbColor::new(0.15, 0.13, 0.11),
    // #F27333 - anchor disc when the tower is tracked.
    accent: RgbColor::new(0.95, 0.45, 0.20),
    // #8C8073 - virtual anchor, landmark rings.
    dim: RgbColor::new(0.55, 0.50, 0.45),
    // #2E526B - halftone Seine wash.
    water: RgbColor::new(0.18, 0.32, 0.42),
    // #F28C26 - selection ring.
    select: RgbColor::new(0.95, 0.55, 0.15),
    // #EBE0CC - cream station discs.
    station: RgbColor::new(0.92, 0.88, 0.80),
    // #E0D9C7 - station labels and hint text.
    label: RgbColor::new(0.88, 0.85, 0.78),
    // #000000 - hint badge fill (drawn at ~0.7 alpha).
    panel: RgbColor::new(0.0, 0.0, 0.0),
    // #F5C518 - taxi yellow.
    taxi: RgbColor::new(0.96, 0.77, 0.09),
    // #2E8C56 - bus green.
    bus: RgbColor::new(0.18, 0.55, 0.34),
    // #3399DC - batobus blue.
    batobus: RgbColor::new(0.20, 0.60, 0.86),
    // #6B5238 - underground secret routes, dashed.
    underground: RgbColor::new(0.42, 0.32, 0.22),
};
