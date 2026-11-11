//! `LabelOrientation` - how a label orients on the tabletop.

use crate::ui::style::renderer::PxPos;
use smearor_commons_geometry::RotationRad;
use std::f64::consts::FRAC_PI_2;

/// How a label orients on the tabletop.
#[derive(Debug, Clone, Copy)]
pub enum LabelOrientation {
    /// Upright in screen space.
    Screen,
    /// Rotated so the text-up faces the map edge nearest the
    /// station - the player on that side reads it upright. The
    /// `PxPos` is the map center in widget px.
    Radial { center: PxPos },
}

impl LabelOrientation {
    /// The label's rotation in screen radians at `pos`: 0 in
    /// `Screen`, facing the nearest map edge in `Radial`.
    pub fn angle(&self, pos: PxPos) -> RotationRad {
        match self {
            Self::Screen => RotationRad::ZERO,
            Self::Radial { center } => RotationRad::new((pos.y - center.y).atan2(pos.x - center.x) - FRAC_PI_2),
        }
    }
}
