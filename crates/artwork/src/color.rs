//! Colour helpers for luminance-based text brightening.

use smearor_commons_color::Color;

/// Computes the perceived luminance (Rec. 601) of a colour in 0..=255.
pub(crate) fn luminance(color: &Color<u8>) -> f64 {
    Color::<f64>::from(*color).luminance_601().as_f64() * 255.0
}

/// Lightens a colour by blending each channel toward white.
pub(crate) fn lighten(color: &Color<u8>, factor: f64) -> Color<u8> {
    Color::<u8>::from(Color::<f64>::from(*color).lighten(factor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luminance_of_black_is_zero() {
        assert_eq!(luminance(&Color::<u8>::BLACK), 0.0);
    }

    #[test]
    fn luminance_of_white_is_255() {
        let result = luminance(&Color::<u8>::WHITE);
        assert!((result - 255.0).abs() < 1e-9);
    }

    #[test]
    fn lighten_brightens_dark_colour() {
        let brown: Color<u8> = "#704B1C".parse().unwrap_or(Color::<u8>::BLACK);
        assert_eq!(lighten(&brown, 0.45).to_string(), "#B09C82");
    }
}
