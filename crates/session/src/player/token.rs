use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fmt::Write as _;

/// Credential issued by `join_game` on the MCP server.
///
/// Scopes agent access to one seat: a detective token sees only the
/// public projection, the fugitive token also sees his hidden position
/// - the same secrecy split as the shield zone. On the wire the
///   token is its 16-hex-digit [`Display`] form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerToken(u64);

impl PlayerToken {
    /// Creates a token from raw bits (the MCP server draws them from a
    /// random source).
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

impl Serialize for PlayerToken {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PlayerToken {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).and_then(|text| text.parse().map_err(serde::de::Error::custom))
    }
}

impl fmt::Display for PlayerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = String::with_capacity(16);
        let _ = write!(buf, "{:016x}", self.0);
        f.write_str(&buf)
    }
}

impl std::str::FromStr for PlayerToken {
    type Err = BadTokenFormat;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(text, 16).map(Self).map_err(|_| BadTokenFormat(text.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_wire_roundtrip() {
        let token = PlayerToken::new(0x36);
        let json = serde_json::to_string(&token).unwrap_or_else(|_| panic!("ser"));
        assert_eq!(json, "\"0000000000000036\"");
        let back: PlayerToken = serde_json::from_str(&json).unwrap_or_else(|_| panic!("de"));
        assert_eq!(back, token);
    }
}

/// Error for a malformed token string (expects 16 hex digits).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("bad player token {0:?}")]
pub struct BadTokenFormat(pub String);
