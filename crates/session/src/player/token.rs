use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fmt::Write as _;

/// Credential issued by `join_game` on the MCP server.
///
/// Scopes agent access to one seat: a detective token sees only the
/// public projection, the Corbeau token also sees his hidden position
/// - the same secrecy split as the shield zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerToken(u64);

impl PlayerToken {
    /// Creates a token from raw bits (the MCP server draws them from a
    /// random source).
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

impl fmt::Display for PlayerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = String::with_capacity(16);
        let _ = write!(buf, "{:016x}", self.0);
        f.write_str(&buf)
    }
}
