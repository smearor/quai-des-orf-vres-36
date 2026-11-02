//! `ArtSpec` - one artwork instance to draw (the `EdgeSpec` for
//! artwork).

use tangible_objects_model::Millimeters;

use crate::ui::art::AnchorFrac;
use crate::ui::art::ArtFamily;
use crate::ui::art::AssetStem;
use crate::ui::style::Alpha;
use crate::ui::style::PxPos;

/// One artwork instance to draw.
pub struct ArtSpec {
    /// The artwork family.
    pub family: ArtFamily,
    /// The asset filename stem.
    pub stem: AssetStem,
    /// Where the document anchor lands, in widget px.
    pub pos: PxPos,
    /// Rendered document height.
    pub height: Millimeters,
    /// Which document fraction sits on `pos`.
    pub anchor: AnchorFrac,
    /// Group opacity.
    pub alpha: Alpha,
}
