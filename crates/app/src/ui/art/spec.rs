//! `ArtSpec` - one artwork instance to draw (the `EdgeSpec` for
//! artwork).

use smearor_commons_color::Alpha;
use tangible_objects_model::Millimeters;
use tangible_objects_model::RotationRad;

use crate::ui::art::AnchorFrac;
use crate::ui::art::ArtFamily;
use crate::ui::art::AssetStem;
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
    /// Rotation around `pos` - the anchor point is the pivot.
    pub rotation: RotationRad,
    /// Group opacity.
    pub alpha: Alpha,
}
