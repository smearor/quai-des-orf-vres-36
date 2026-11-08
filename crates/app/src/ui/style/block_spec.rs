//! `BlockSpec` - one city block the scene draws.

use crate::ui::style::DistrictLabel;
use crate::ui::style::HouseMode;
use crate::ui::style::LabelOrientation;
use crate::ui::style::ParkSpot;
use crate::ui::style::PxPos;
use crate::ui::style::ToPx;
use qdo36_map::House;
use std::f64::consts::FRAC_1_SQRT_2;
use tangible_objects_model::Pixels;
use tangible_objects_model::PixelsPerMm;

/// One city block: its inset outline plus the procedural houses
/// inside, all in map mm.
pub struct BlockSpec<'a> {
    /// The block outline, already inset from its bounding streets,
    /// river and border.
    pub outline: &'a qdo36_map::Polygon,
    /// Houses inside the block - quads fronting the outline ring
    /// plus rounded corner footprints.
    pub buildings: &'a [House],
    /// Decorative streets crossing the block interior (visual
    /// only - the houses stay out of their corridors).
    pub streets: &'a [qdo36_map::Polyline],
    /// Park blobs inside the block, kept free of houses.
    pub parks: &'a [ParkSpot],
    /// The district name the block carries, drawn at the deepest
    /// inside point.
    pub label: Option<DistrictLabel>,
    /// District caption orientation (radial faces the nearest
    /// table edge, screen stays upright).
    pub district_orientation: LabelOrientation,
    /// Park caption orientation.
    pub park_orientation: LabelOrientation,
    /// How the houses render (top-down footprints, fixed or
    /// radial isometric extrusion).
    pub house_mode: HouseMode,
    /// Map center in widget px - the radial extrusion pivot.
    pub center: PxPos,
    /// Pixel scale for stroke widths.
    pub px_per_mm: PixelsPerMm,
}

impl BlockSpec<'_> {
    /// The block's house extrusion shift for `height` (widget
    /// px): flat top-down blocks don't shift, classic iso shifts
    /// up-left, radial iso shifts each block's outline centroid
    /// toward `center` so walls face the nearest table edge.
    pub fn extrusion_shift(&self, height: Pixels, to_px: ToPx<'_>) -> PxPos {
        match self.house_mode {
            HouseMode::TopDown => PxPos::ZERO,
            HouseMode::Iso => {
                let d = -height * FRAC_1_SQRT_2;
                PxPos::from_px(d, d)
            }
            HouseMode::RadialIso => self.radial_shift(height, to_px),
        }
    }

    /// The radial extrusion: this block's outline centroid to the
    /// map center, scaled to `height` - walls spill outward toward
    /// the nearest table edge. A degenerate block at the center
    /// falls back to the fixed iso direction.
    fn radial_shift(&self, height: Pixels, to_px: ToPx<'_>) -> PxPos {
        let c = to_px.pos(self.outline.centroid());
        let (dx, dy) = (self.center.x - c.x, self.center.y - c.y);
        let len = dx.hypot(dy);
        if len < 1.0 {
            let d = -height * FRAC_1_SQRT_2;
            return PxPos::from_px(d, d);
        }
        PxPos::from_px(height * (dx / len), height * (dy / len))
    }
}
