use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error as _;
use smearor_commons_color::Color;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// A transit line's brand color (RATP/BVG palette).
///
/// TOML shape `"#RRGGBB"`; exposes `rgb_u8()` for cairo rendering.
/// Thin wrapper around [`Color<u8>`] with a strictly 6-digit wire
/// format (no shorthand, no alpha channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineColor(Color<u8>);

impl LineColor {
    /// The RGB channels.
    pub fn rgb_u8(&self) -> [u8; 3] {
        [self.0.r, self.0.g, self.0.b]
    }

    /// The color as a [`Color<u8>`] (alpha always 255).
    pub fn color(&self) -> Color<u8> {
        self.0
    }
}

impl fmt::Display for LineColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.0.r, self.0.g, self.0.b)
    }
}

impl PartialOrd for LineColor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LineColor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rgb_u8().cmp(&other.rgb_u8())
    }
}

impl FromStr for LineColor {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let hex = text.strip_prefix('#').ok_or_else(|| format!("line color '{text}' must start with '#'"))?;
        if hex.len() != 6 {
            return Err(format!("line color '{text}' is not #RRGGBB"));
        }
        text.parse::<Color<u8>>().map(Self).map_err(|_| format!("line color '{text}' is not #RRGGBB"))
    }
}

impl Serialize for LineColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LineColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(D::Error::custom)
    }
}
