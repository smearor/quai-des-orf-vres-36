use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Color of a detective pawn.
///
/// Shared vocabulary across physical pawns, rendered markers and seat
/// assignment. The canonical palette lives in
/// `concepts/planned/ARTWORK.md` (style-book §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PawnColor {
    /// Blue detective.
    Blue,
    /// Red detective.
    Red,
    /// Green detective.
    Green,
    /// Yellow detective.
    Yellow,
    /// Purple detective.
    Purple,
}

impl PawnColor {
    /// All pawn colors in seat order.
    pub const ALL: [PawnColor; 5] = [Self::Blue, Self::Red, Self::Green, Self::Yellow, Self::Purple];

    /// The canonical RGB value from the style-book.
    pub const fn rgb_u8(self) -> (u8, u8, u8) {
        match self {
            Self::Blue => (0x25, 0x63, 0xEB),
            Self::Red => (0xDC, 0x26, 0x26),
            Self::Green => (0x16, 0xA3, 0x4A),
            Self::Yellow => (0xEA, 0xB3, 0x08),
            Self::Purple => (0x93, 0x33, 0xEA),
        }
    }
}

impl fmt::Display for PawnColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Blue => "blue",
            Self::Red => "red",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Purple => "purple",
        };
        f.write_str(name)
    }
}
