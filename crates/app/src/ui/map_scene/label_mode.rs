//! `LabelMode` - how station labels orient on the tabletop.

use crate::ui::style::LabelOrientation;
use crate::ui::style::PxPos;
use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::RotationRad;
use std::f64::consts::FRAC_PI_2;
use std::str::FromStr;

/// Label orientation on the tabletop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LabelMode {
    /// Each label faces the nearest table edge - readable for the
    /// player on that side (tabletop default).
    #[default]
    Radial,
    /// Every label upright in screen space.
    Screen,
}

impl LabelMode {
    /// The caption orientation this mode produces at the map
    /// `center`: radial pivots on it, screen stays upright.
    pub fn orientation(self, center: PxPos) -> LabelOrientation {
        match self {
            Self::Radial => LabelOrientation::Radial { center },
            Self::Screen => LabelOrientation::Screen,
        }
    }

    /// The rotation this mode prefers for a label at `pos` -
    /// screen-upright or radially outward from `center`.
    pub fn preferred(self, pos: PxPos, center: PxPos) -> RotationRad {
        match self {
            Self::Screen => RotationRad::new(-FRAC_PI_2),
            Self::Radial => (pos - center).direction(),
        }
    }
}

impl FromStr for LabelMode {
    type Err = UnknownLabelMode;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "radial" => Ok(Self::Radial),
            "screen" => Ok(Self::Screen),
            other => Err(UnknownLabelMode(other.to_string())),
        }
    }
}

/// Error for an unrecognized label-mode name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown label_mode {0:?}")]
pub struct UnknownLabelMode(pub String);
