use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error as _;
use std::fmt;

/// A transit line's brand color (RATP/BVG palette).
///
/// TOML shape `"#RRGGBB"`; exposes `rgb_u8()` for cairo rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineColor([u8; 3]);

impl LineColor {
    /// The RGB channels.
    pub fn rgb_u8(&self) -> [u8; 3] {
        self.0
    }
}

impl fmt::Display for LineColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.0[0], self.0[1], self.0[2])
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
        let hex = text
            .strip_prefix('#')
            .ok_or_else(|| D::Error::custom(format!("line color '{text}' must start with '#'")))?;
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(D::Error::custom(format!("line color '{text}' is not #RRGGBB")));
        }
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(D::Error::custom);
        Ok(Self([channel(0)?, channel(2)?, channel(4)?]))
    }
}
