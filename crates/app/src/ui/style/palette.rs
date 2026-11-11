//! `StylePalette` - the semantic color tokens a map style carries.
//!
//! The slot names mirror the SVG style-book tokens (`ARTWORK.md`
//! §2: bg, ink, accent, water, taxi, bus, batobus, station,
//! highlight, panel_bg) so the cairo renderers speak the same
//! vocabulary as the generated artwork. Metro edges are
//! deliberately absent - their colors come from `map.meta().lines`,
//! not from the style.

use crate::ui::style::StyleColor;
use qdo36_map::GameMap;
use qdo36_map::LineName;
use qdo36_map::Transport;
use smearor_commons_color::Color;
use smearor_commons_color::Palette;
use std::ops::Deref;

/// One style's semantic colors - a [`Palette`] keyed by
/// [`StyleColor`].
#[derive(Debug, Clone)]
pub struct StylePalette {
    /// The registered slot colors.
    colors: Palette<StyleColor, f64>,
}

impl StylePalette {
    /// The color registered under `slot` - `Color::BLACK` when the
    /// slot carries no color.
    pub fn color(&self, slot: StyleColor) -> Color<f64> {
        self.colors.get_or(&slot, Color::<f64>::BLACK)
    }

    /// The field the scene paints on.
    pub fn paper(&self) -> Color<f64> {
        self.color(StyleColor::Paper)
    }

    /// Primary foreground ink: station outlines, labels, hairlines.
    pub fn ink(&self) -> Color<f64> {
        self.color(StyleColor::Ink)
    }

    /// Signature accent: gold, neon glow, comic ink-pop.
    pub fn accent(&self) -> Color<f64> {
        self.color(StyleColor::Accent)
    }

    /// Dimmed secondary: virtual anchor, muted elements.
    pub fn dim(&self) -> Color<f64> {
        self.color(StyleColor::Dim)
    }

    /// Water accent - the river's two bank lines.
    pub fn water(&self) -> Color<f64> {
        self.color(StyleColor::Water)
    }

    /// The river body between the bank lines - darker than
    /// `water`, distinct from `paper`.
    pub fn water_fill(&self) -> Color<f64> {
        self.color(StyleColor::WaterFill)
    }

    /// Selection highlight ring.
    pub fn select(&self) -> Color<f64> {
        self.color(StyleColor::Select)
    }

    /// Station node fill.
    pub fn station(&self) -> Color<f64> {
        self.color(StyleColor::Station)
    }

    /// Label and hint text.
    pub fn label(&self) -> Color<f64> {
        self.color(StyleColor::Label)
    }

    /// Hint/overlay box fill.
    pub fn panel(&self) -> Color<f64> {
        self.color(StyleColor::Panel)
    }

    /// Taxi route color.
    pub fn taxi(&self) -> Color<f64> {
        self.color(StyleColor::Taxi)
    }

    /// Bus route color.
    pub fn bus(&self) -> Color<f64> {
        self.color(StyleColor::Bus)
    }

    /// Batobus route color.
    pub fn batobus(&self) -> Color<f64> {
        self.color(StyleColor::Batobus)
    }

    /// Underground secret-route color.
    pub fn underground(&self) -> Color<f64> {
        self.color(StyleColor::Underground)
    }

    /// Generic metro color for unbranded metro marks (station
    /// rings); edges use the per-line `[map.lines]` color instead.
    pub fn metro(&self) -> Color<f64> {
        self.color(StyleColor::Metro)
    }

    /// Neutral edge ink - the default web color when a transport
    /// layer is not highlighted (standard view).
    pub fn edge(&self) -> Color<f64> {
        self.color(StyleColor::Edge)
    }

    /// City-block outline - the free areas between streets, river
    /// and border.
    pub fn block(&self) -> Color<f64> {
        self.color(StyleColor::Block)
    }

    /// Procedural building fill inside a block - slightly raised
    /// from `paper`.
    pub fn building(&self) -> Color<f64> {
        self.color(StyleColor::Building)
    }

    /// Park green inside a block - the dataset's named green
    /// spots.
    pub fn park(&self) -> Color<f64> {
        self.color(StyleColor::Park)
    }

    /// Roof detail lines inside a house footprint - contrast
    /// against `building`, not `block`.
    pub fn roof(&self) -> Color<f64> {
        self.color(StyleColor::Roof)
    }

    /// Decorative streets crossing a block interior - muted, below
    /// the transport colors.
    pub fn street(&self) -> Color<f64> {
        self.color(StyleColor::Street)
    }

    /// The transport's own palette color (no line lookup) - the
    /// ring color of a served station.
    pub fn base_color(&self, transport: Transport) -> Color<f64> {
        self.color(match transport {
            Transport::Taxi => StyleColor::Taxi,
            Transport::Bus => StyleColor::Bus,
            Transport::Batobus => StyleColor::Batobus,
            Transport::Underground => StyleColor::Underground,
            Transport::Metro => StyleColor::Metro,
        })
    }

    /// Edge color per transport: fixed routes come from the palette,
    /// metro edges carry their declared line color from `map.lines`.
    pub fn transport_color(&self, transport: Transport, line: Option<&LineName>, map: &GameMap) -> Color<f64> {
        match transport {
            Transport::Metro => line
                .and_then(|name| map.meta().lines.get(name))
                .map(|color| {
                    let [r, g, b] = color.rgb_u8();
                    Color::<u8>::new(r, g, b).into()
                })
                .unwrap_or(self.metro()),
            other => self.base_color(other),
        }
    }
}

/// A [`StylePalette`] is built from its slot colors.
impl From<Palette<StyleColor, f64>> for StylePalette {
    fn from(colors: Palette<StyleColor, f64>) -> Self {
        Self { colors }
    }
}

/// The wrapped [`Palette`] - `get`, `iter`, `missing`,
/// `is_complete`.
impl Deref for StylePalette {
    type Target = Palette<StyleColor, f64>;

    fn deref(&self) -> &Self::Target {
        &self.colors
    }
}
