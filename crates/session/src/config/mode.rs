use serde::Deserialize;
use serde::Serialize;

/// How players place their pawns, mirroring tangibles-chess's input
/// modes.
///
/// The Eiffel tower anchor is an independent mechanic: in
/// `NoTangibles` it is substituted by a virtual anchor pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TangibleMode {
    /// Touch input only: detectives tap stations, pawns are rendered,
    /// Corbeau uses a hidden fallback dialog, the anchor is virtual.
    NoTangibles,
    /// Physical pawns plus a rendered marker layer with discrepancy
    /// display ("physical & rendering" in tangibles-chess).
    TangiblesRendered,
    /// Physical pawns only; the app shows guidance highlights but no
    /// rendered pawn markers ("physical only" in tangibles-chess).
    Tangibles,
}

impl TangibleMode {
    /// Whether pawn patterns and field sets are registered.
    pub fn uses_tangibles(self) -> bool {
        !matches!(self, Self::NoTangibles)
    }

    /// Whether pawn markers are rendered on the map.
    pub fn renders_pawns(self) -> bool {
        !matches!(self, Self::Tangibles)
    }
}
