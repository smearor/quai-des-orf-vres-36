//! Colour helpers for luminance-based text brightening.

use crate::error::ArtworkError;

/// Parses a `#rrggbb` hex colour literal into its RGB channels.
fn parse_hex_colour(value: &str) -> Result<(u8, u8, u8), ArtworkError> {
    let hex = value.strip_prefix('#').ok_or_else(|| ArtworkError::InvalidColour(value.to_owned()))?;
    if hex.len() != 6 {
        return Err(ArtworkError::InvalidColour(value.to_owned()));
    }
    let channel = |index: usize| u8::from_str_radix(&hex[index..index + 2], 16).map_err(|_| ArtworkError::InvalidColour(value.to_owned()));
    Ok((channel(0)?, channel(2)?, channel(4)?))
}

/// Computes the perceived luminance (Rec. 601) of a `#rrggbb` colour.
pub(crate) fn luminance(value: &str) -> Result<f64, ArtworkError> {
    let (red, green, blue) = parse_hex_colour(value)?;
    Ok(0.299 * f64::from(red) + 0.587 * f64::from(green) + 0.114 * f64::from(blue))
}

/// Lightens a `#rrggbb` colour by blending each channel toward white.
pub(crate) fn lighten(value: &str, factor: f64) -> Result<String, ArtworkError> {
    let (red, green, blue) = parse_hex_colour(value)?;
    let blend = |channel: u8| -> String {
        let channel = f64::from(channel);
        format!("{:02X}", (channel + (255.0 - channel) * factor).round().min(255.0) as u8)
    };
    Ok(format!("#{}{}{}", blend(red), blend(green), blend(blue)))
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
