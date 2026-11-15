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
    pub width: Pixels,
    /// Pixel height.
    pub height: Pixels,
}
