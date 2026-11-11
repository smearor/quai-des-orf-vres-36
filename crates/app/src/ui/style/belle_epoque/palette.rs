//! The Belle Époque style palette (ARTWORK §2).

use crate::ui::style::StylePalette;
use smearor_commons_color::Color;

/// Parchment field, sepia engraving ink, muted gold accents - the
/// same voice as the generated masters.
pub(crate) const PALETTE: StylePalette = StylePalette {
    // #F7EBD0 - parchment field.
    paper: Color::<f64>::new(0.97, 0.92, 0.81),
    // #542C15 - sepia engraving ink: outlines, hairlines, anchor disc.
    ink: Color::<f64>::new(0.33, 0.17, 0.08),
    // #B8871C - muted gold: ornamental rings, anchor ring, hint frame.
    accent: Color::<f64>::new(0.72, 0.53, 0.11),
    // #8C734C - faded sepia for the virtual anchor.
    dim: Color::<f64>::new(0.55, 0.45, 0.30),
    // #9EB8C7 - pale engraved Seine wash.
    water: Color::<f64>::new(0.62, 0.72, 0.78),
    // #94ADBB - river body, a touch deeper than the bank wash.
    water_fill: Color::<f64>::new(0.58, 0.68, 0.73),
    // #B8871C - gold selection ring (the same accent tone).
    select: Color::<f64>::new(0.72, 0.53, 0.11),
    // #FAF2DB - station disc fill, lighter than the paper.
    station: Color::<f64>::new(0.98, 0.95, 0.86),
    // #542C15 - sepia labels and hint text.
    label: Color::<f64>::new(0.33, 0.17, 0.08),
    // #F7EBD0 - parchment hint badge (drawn at ~0.94 alpha).
    panel: Color::<f64>::new(0.97, 0.92, 0.81),
    // #C79914 - taxi, muted ochre.
    taxi: Color::<f64>::new(0.78, 0.60, 0.08),
    // #386B48 - bus, period green.
    bus: Color::<f64>::new(0.22, 0.42, 0.28),
    // #2E7385 - batobus, faded teal.
    batobus: Color::<f64>::new(0.18, 0.45, 0.52),
    // #542C15 - underground secret routes, sepia dashes.
    underground: Color::<f64>::new(0.33, 0.17, 0.08),
    // #6E6252 - neutral edge ink in sepia (standard view).
    edge: Color::<f64>::new(0.43, 0.38, 0.32),
    // #6B4A8E - period mauve: generic metro ring mark and the
    // metro lane in parallel-transports view.
    metro: Color::<f64>::new(0.42, 0.29, 0.56),
    // #C9B98F - faded sepia: city-block outlines on the
    // parchment field.
    block: Color::<f64>::new(0.79, 0.73, 0.56),
    // #E4D3A9 - aged-stone fill for the procedural houses.
    building: Color::<f64>::new(0.89, 0.83, 0.66),
    park: Color::<f64>::new(0.62, 0.72, 0.50),
    // #A98F61 - engraved sepia roof lines, darker than the
    // building fill.
    roof: Color::<f64>::new(0.66, 0.56, 0.38),
    // #8F7D5C - quiet sepia for decorative streets.
    street: Color::<f64>::new(0.56, 0.49, 0.36),
};
