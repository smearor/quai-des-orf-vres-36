//! `ArtSpec` - one artwork instance to draw (the `EdgeSpec` for
//! artwork).

use tangible_objects_model::Millimeters;

use crate::ui::art::AnchorFrac;
use crate::ui::style::Alpha;
use crate::ui::style::PxPos;

/// One artwork instance to draw.
pub struct ArtSpec<'a> {
    /// The artwork family directory (`landmarks`, `map`, ...).
    pub family: &'a str,
    /// The asset filename without extension.
    pub stem: &'a str,
    /// Where the document anchor lands, in widget px.
    pub pos: PxPos,
    /// Rendered document height.
    pub height: Millimeters,
    /// Which document fraction sits on `pos`.
    pub anchor: AnchorFrac,
    /// Group opacity.
    pub alpha: Alpha,
}
