//! `LabelSpec` - everything a style needs to draw one text label.

use crate::ui::style::LabelOrientation;
use crate::ui::style::PxPos;
use qdo36_tangibles::ScaleFactor;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;

/// A text label attached to a station - name below the disc, number
/// inside it, score further out.
#[derive(Debug)]
pub struct LabelSpec<'a> {
    /// The text to draw.
    pub text: &'a str,
    /// Station center in widget px.
    pub pos: PxPos,
    /// Station disc radius - the outside gap derives from it.
    pub radius: Pixels,
    /// Orientation on the tabletop (screen vs radial).
    pub orientation: LabelOrientation,
    /// Center the text on `pos` (inside the disc) instead of the
    /// outside gap.
    pub inside: bool,
    /// Global font size factor (`RenderParams.label_scale`).
    pub label_scale: ScaleFactor,
    /// Pixels per surface millimeter.
    pub px_per_mm: PixelsPerMm,
}
