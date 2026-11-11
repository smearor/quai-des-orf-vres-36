//! `ScreenshotReply` - the window PNG answer of the `screenshot`
//! command.

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Pixels;

/// A rendered app-window frame (`screenshot` tool/command).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotReply {
    /// PNG image, base64-encoded.
    pub png_base64: String,
    /// Pixel width.
    #[serde(with = "pixels_wire")]
    pub width: Pixels,
    /// Pixel height.
    #[serde(with = "pixels_wire")]
    pub height: Pixels,
}

/// `Pixels` is wire-plain `f64`; upstream carries no serde derives.
mod pixels_wire {
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;
    use smearor_commons_geometry::Pixels;

    /// Pixel value as a plain JSON number.
    pub fn serialize<S: Serializer>(pixels: &Pixels, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(pixels.as_f64())
    }

    /// Reads a plain JSON number into `Pixels`.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Pixels, D::Error> {
        f64::deserialize(deserializer).map(Pixels::new)
    }
}
