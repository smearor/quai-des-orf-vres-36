//! `HouseMode` - how procedural houses render on the tabletop.

use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;

/// House rendering mode on the tabletop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HouseMode {
    /// Each block's houses extrude toward the table edge nearest
    /// its centroid - the player on that side sees their fronts
    /// (tabletop default).
    #[default]
    RadialIso,
    /// All houses extrude in one fixed direction (up-left on
    /// screen - walls face the bottom-right).
    Iso,
    /// Plain footprints with hip-roof details, no extrusion.
    TopDown,
}

impl HouseMode {
    /// Every mode the orientation menu offers.
    pub const ALL: [HouseMode; 3] = [HouseMode::RadialIso, HouseMode::Iso, HouseMode::TopDown];
}

impl FromStr for HouseMode {
    type Err = UnknownHouseMode;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "radial-iso" => Ok(Self::RadialIso),
            "iso" => Ok(Self::Iso),
            "top-down" => Ok(Self::TopDown),
            other => Err(UnknownHouseMode(other.to_string())),
        }
    }
}

/// Error for an unrecognized house-mode name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown house_mode {0:?}")]
pub struct UnknownHouseMode(pub String);
