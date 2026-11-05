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

use crate::ui::style::RgbColor;

/// One style's semantic colors.
#[derive(Debug, Clone, Copy)]
pub struct StylePalette {
    /// The field the scene paints on.
    pub paper: RgbColor,
    /// Primary foreground ink: station outlines, labels, hairlines.
    pub ink: RgbColor,
    /// Signature accent: gold, neon glow, comic ink-pop.
    pub accent: RgbColor,
    /// Dimmed secondary: virtual anchor, muted elements.
    pub dim: RgbColor,
    /// Water accent - the river's two bank lines.
    pub water: RgbColor,
    /// The river body between the bank lines - darker than `water`,
    /// distinct from `paper`.
    pub water_fill: RgbColor,
    /// Selection highlight ring.
    pub select: RgbColor,
    /// Station node fill.
    pub station: RgbColor,
    /// Label and hint text.
    pub label: RgbColor,
    /// Hint/overlay box fill.
    pub panel: RgbColor,
    /// Taxi route color.
    pub taxi: RgbColor,
    /// Bus route color.
    pub bus: RgbColor,
    /// Batobus route color.
    pub batobus: RgbColor,
    /// Underground secret-route color.
    pub underground: RgbColor,
    /// Generic metro color for unbranded metro marks (station rings);
    /// edges use the per-line `[map.lines]` color instead.
    pub metro: RgbColor,
    /// Neutral edge ink - the default web color when a transport
    /// layer is not highlighted (standard view).
    pub edge: RgbColor,
}

impl StylePalette {
    /// The transport's own palette color (no line lookup) - the
    /// ring color of a served station.
    pub fn base_color(&self, transport: Transport) -> RgbColor {
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
    pub fn transport_color(&self, transport: Transport, line: Option<&LineName>, map: &GameMap) -> RgbColor {
        match transport {
            Transport::Metro => line
                .and_then(|name| map.meta().lines.get(name))
                .map(|color| {
                    let [r, g, b] = color.rgb_u8();
                    RgbColor::from_u8(r, g, b)
                })
                .unwrap_or(self.metro),
            other => self.base_color(other),
        }
    }
}
