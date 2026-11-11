//! Colour helpers for luminance-based text brightening.

use crate::error::ArtworkError;
use smearor_commons_color::Color;
use std::str::FromStr;

/// Parses a `#rrggbb` hex colour literal.
fn parse_hex_colour(value: &str) -> Result<Color<u8>, ArtworkError> {
    let hex = value.strip_prefix('#').ok_or_else(|| ArtworkError::InvalidColour(value.to_owned()))?;
    if hex.len() != 6 {
        return Err(ArtworkError::InvalidColour(value.to_owned()));
    }
    Color::from_str(value).map_err(|_| ArtworkError::InvalidColour(value.to_owned()))
}

/// Computes the perceived luminance (Rec. 601) of a `#rrggbb` colour.
pub(crate) fn luminance(value: &str) -> Result<f64, ArtworkError> {
    let color = parse_hex_colour(value)?;
    Ok(Color::<f64>::from(color).luminance_601() * 255.0)
}

/// Lightens a `#rrggbb` colour by blending each channel toward white.
pub(crate) fn lighten(value: &str, factor: f64) -> Result<String, ArtworkError> {
    let color = parse_hex_colour(value)?;
    Ok(Color::<u8>::from(Color::<f64>::from(color).lighten(factor)).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luminance_of_black_is_zero() {
        assert_eq!(luminance("#000000").unwrap_or(-1.0), 0.0);
    }

    #[test]
    fn luminance_of_white_is_255() {
        let result = luminance("#FFFFFF").unwrap_or(-1.0);
        assert!((result - 255.0).abs() < 1e-9);
    }

    #[test]
    fn lighten_brightens_dark_colour() {
        assert_eq!(lighten("#704B1C", 0.45).unwrap_or_default(), "#B09C82");
    }

    #[test]
    fn lighten_rejects_invalid_colour() {
        assert!(matches!(lighten("704B1C", 0.45), Err(ArtworkError::InvalidColour(_))));
    }
}
