//! `StylePalette` - the semantic color tokens a map style carries.
//!
//! The field names mirror the SVG style-book tokens (`ARTWORK.md` §2:
//! bg, ink, accent, water, taxi, bus, batobus, station, highlight,
//! panel_bg) so the cairo renderers speak the same vocabulary as the
//! generated artwork. Metro edges are deliberately absent - their
//! colors come from `map.meta().lines`, not from the style.

use qdo36_map::GameMap;
use qdo36_map::LineName;
use qdo36_map::Transport;
use smearor_commons_color::Color;

/// One style's semantic colors.
#[derive(Debug, Clone, Copy)]
pub struct StylePalette {
    /// The field the scene paints on.
    pub paper: Color<f64>,
    /// Primary foreground ink: station outlines, labels, hairlines.
    pub ink: Color<f64>,
    /// Signature accent: gold, neon glow, comic ink-pop.
    pub accent: Color<f64>,
    /// Dimmed secondary: virtual anchor, muted elements.
    pub dim: Color<f64>,
    /// Water accent - the river's two bank lines.
    pub water: Color<f64>,
    /// The river body between the bank lines - darker than `water`,
    /// distinct from `paper`.
    pub water_fill: Color<f64>,
    /// Selection highlight ring.
    pub select: Color<f64>,
    /// Station node fill.
    pub station: Color<f64>,
    /// Label and hint text.
    pub label: Color<f64>,
    /// Hint/overlay box fill.
    pub panel: Color<f64>,
    /// Taxi route color.
    pub taxi: Color<f64>,
    /// Bus route color.
    pub bus: Color<f64>,
    /// Batobus route color.
    pub batobus: Color<f64>,
    /// Underground secret-route color.
    pub underground: Color<f64>,
    /// Generic metro color for unbranded metro marks (station rings);
    /// edges use the per-line `[map.lines]` color instead.
    pub metro: Color<f64>,
    /// Neutral edge ink - the default web color when a transport
    /// layer is not highlighted (standard view).
    pub edge: Color<f64>,
    /// City-block outline - the free areas between streets, river
    /// and border.
    pub block: Color<f64>,
    /// Procedural building fill inside a block - slightly raised
    /// from `paper`.
    pub building: Color<f64>,
    /// Park green inside a block - the dataset's named green
    /// spots.
    pub park: Color<f64>,
    /// Roof detail lines inside a house footprint - contrast
    /// against `building`, not `block`.
    pub roof: Color<f64>,
    /// Decorative streets crossing a block interior - muted, below
    /// the transport colors.
    pub street: Color<f64>,
}

impl StylePalette {
    /// The transport's own palette color (no line lookup) - the
    /// ring color of a served station.
    pub fn base_color(&self, transport: Transport) -> Color<f64> {
        match transport {
            Transport::Taxi => self.taxi,
            Transport::Bus => self.bus,
            Transport::Batobus => self.batobus,
            Transport::Underground => self.underground,
            Transport::Metro => self.metro,
        }
    }

    /// Edge color per transport: fixed routes come from the palette,
    /// metro edges carry their declared line color from `map.lines`.
    pub fn transport_color(&self, transport: Transport, line: Option<&LineName>, map: &GameMap) -> Color<f64> {
        match transport {
            Transport::Metro => line
                .and_then(|name| map.meta().lines.get(name))
                .map(|color| {
                    let [r, g, b] = color.rgb_u8();
                    smearor_commons_color::Color::<u8>::new(r, g, b).into()
                })
                .unwrap_or(self.metro),
            other => self.base_color(other),
        }
    }
}
