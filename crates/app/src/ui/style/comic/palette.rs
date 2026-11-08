//! The Comic style palette (ARTWORK §2).

use crate::ui::style::RgbColor;
use crate::ui::style::StylePalette;

/// Flat fills on a dark comic-board field, cream station discs,
/// orange accents.
pub(crate) const PALETTE: StylePalette = StylePalette {
    // #1A1714 - dark field the board paints on.
    paper: RgbColor::new(0.10, 0.09, 0.08),
    // #26211C - node outlines.
    ink: RgbColor::new(0.15, 0.13, 0.11),
    // #F27333 - anchor disc when the tower is tracked.
    accent: RgbColor::new(0.95, 0.45, 0.20),
    // #8C8073 - virtual anchor, landmark rings.
    dim: RgbColor::new(0.55, 0.50, 0.45),
    // #2E526B - halftone Seine bank lines.
    water: RgbColor::new(0.18, 0.32, 0.42),
    // #0F1C29 - dark river body, cooler than the warm field.
    water_fill: RgbColor::new(0.06, 0.11, 0.16),
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
    // #4A443C - neutral edge ink (standard view, no highlight).
    edge: RgbColor::new(0.29, 0.27, 0.24),
    // #7C3AED - bold purple: generic metro ring mark and the
    // metro lane in parallel-transports view.
    metro: RgbColor::new(0.49, 0.23, 0.93),
    // #3D3229 - quiet warm brown: city-block outlines over the
    // dark field.
    block: RgbColor::new(0.24, 0.20, 0.16),
    // #4A3B28 - warm brick fill for the procedural houses.
    building: RgbColor::new(0.29, 0.23, 0.16),
    park: RgbColor::new(0.48, 0.58, 0.30),
    // #9E8B64 - light sepia roof lines on the dark building fill.
    roof: RgbColor::new(0.62, 0.55, 0.39),
    // #6B5F4C - quiet warm grey for decorative streets.
    street: RgbColor::new(0.42, 0.37, 0.30),
};
