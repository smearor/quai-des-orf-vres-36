//! `LabelMode` - how station labels orient on the tabletop.

use std::str::FromStr;

/// Label orientation on the tabletop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelMode {
    /// Each label faces the nearest table edge - readable for the
    /// player on that side (tabletop default).
    #[default]
    Radial,
    /// Every label upright in screen space.
    Screen,
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
