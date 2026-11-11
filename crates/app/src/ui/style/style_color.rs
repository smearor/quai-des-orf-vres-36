//! `StyleColor` - the semantic color slots a map style declares.
//!
//! The slot names mirror the SVG style-book tokens (`ARTWORK.md`
//! §2: bg, ink, accent, water, taxi, bus, batobus, station,
//! highlight, panel_bg) so the cairo renderers speak the same
//! vocabulary as the generated artwork. Metro edges are
//! deliberately absent - their colors come from `map.meta().lines`,
//! not from the style.

use smearor_commons_color::palette_definition;

palette_definition! {
    /// The semantic color slots of a map style - the key type of
    /// [`crate::ui::style::StylePalette`].
    pub enum StyleColor {
        /// The field the scene paints on.
        Paper => "paper",
        /// Primary foreground ink: station outlines, labels, hairlines.
        Ink => "ink",
        /// Signature accent: gold, neon glow, comic ink-pop.
        Accent => "accent",
        /// Dimmed secondary: virtual anchor, muted elements.
        Dim => "dim",
        /// Water accent - the river's two bank lines.
        Water => "water",
        /// The river body between the bank lines - darker than
        /// `Water`, distinct from `Paper`.
        WaterFill => "water_fill",
        /// Selection highlight ring.
        Select => "select",
        /// Station node fill.
        Station => "station",
        /// Label and hint text.
        Label => "label",
        /// Hint/overlay box fill.
        Panel => "panel",
        /// Taxi route color.
        Taxi => "taxi",
        /// Bus route color.
        Bus => "bus",
        /// Batobus route color.
        Batobus => "batobus",
        /// Underground secret-route color.
        Underground => "underground",
        /// Generic metro color for unbranded metro marks (station
        /// rings); edges use the per-line `[map.lines]` color
        /// instead.
        Metro => "metro",
        /// Neutral edge ink - the default web color when a
        /// transport layer is not highlighted (standard view).
        Edge => "edge",
        /// City-block outline - the free areas between streets,
        /// river and border.
        Block => "block",
        /// Procedural building fill inside a block - slightly
        /// raised from `Paper`.
        Building => "building",
        /// Park green inside a block - the dataset's named green
        /// spots.
        Park => "park",
        /// Roof detail lines inside a house footprint - contrast
        /// against `Building`, not `Block`.
        Roof => "roof",
        /// Decorative streets crossing a block interior - muted,
        /// below the transport colors.
        Street => "street"
    }
}
