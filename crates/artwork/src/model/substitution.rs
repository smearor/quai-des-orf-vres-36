//! Attribute substitution declaration.

use crate::color;
use serde::Deserialize;
use smearor_commons_color::Color;

/// Default blend factor toward white for luminance-based lightening.
const DEFAULT_LIGHTEN_FACTOR: f64 = 0.45;

/// One attribute substitution on a parametrized element.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Substitution {
    /// Id of the parametrized element in the master.
    pub id: String,
    /// Attribute whose value is replaced (for example `stop-color`).
    pub attribute: String,
    /// `#rrggbb` colour value written to the attribute.
    pub value: Color<u8>,
    /// Optional luminance threshold; colours darker than this are lightened
    /// by `lighten_factor` instead of being written verbatim.
    #[serde(default)]
    pub lighten_if_below: Option<f64>,
    /// Blend factor toward white applied by `lighten_if_below` (0.0..=1.0).
    #[serde(default = "default_lighten_factor")]
    pub lighten_factor: f64,
}

/// Serde default for [`Substitution::lighten_factor`].
fn default_lighten_factor() -> f64 {
    DEFAULT_LIGHTEN_FACTOR
}

impl Substitution {
    /// Resolves the effective attribute colour, applying the optional
    /// luminance-based lightening for dark colours.
    pub fn resolve(&self) -> Color<u8> {
        match self.lighten_if_below {
            Some(threshold) if color::luminance(&self.value) < threshold => color::lighten(&self.value, self.lighten_factor),
            _ => self.value,
        }
    }
}
