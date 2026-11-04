//! `LayerName` - the identifier of one scene layer.

use std::fmt;

/// A scene layer - the MCP layer keys, the overlay checkbox
/// captions and the `SceneLayers` fields share this identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerName {
    /// Taxi edges.
    Taxi,
    /// Bus edges.
    Bus,
    /// Metro edges (+ the metro overlay extras).
    Metro,
    /// The metro overlay itself - highlighted lines, parallel
    /// offsets, badges and pie-filled transfer stations.
    MetroOverlay,
    /// Underground edges.
    Underground,
    /// Batobus edges.
    Batobus,
    /// Water backdrop.
    Water,
    /// Landmark markers.
    Landmarks,
    /// Station discs + service rings.
    Stations,
    /// Station name labels.
    Labels,
    /// In-disc station numbers.
    Numbers,
    /// Strategic importance scores.
    Scores,
    /// The compass rose.
    Compass,
    /// Bridge-station markers - dataset debugging only, toggled
    /// via MCP (kept out of `ALL`, so no overlay checkbox).
    Bridges,
}

impl LayerName {
    /// Every toggleable layer, in overlay-menu order.
    pub const ALL: &[LayerName] = &[
        Self::Taxi,
        Self::Bus,
        Self::Metro,
        Self::MetroOverlay,
        Self::Underground,
        Self::Batobus,
        Self::Water,
        Self::Landmarks,
        Self::Stations,
        Self::Labels,
        Self::Numbers,
        Self::Scores,
        Self::Compass,
    ];

    /// The layer's stable key (`"taxi"`, `"labels"`, ...) - MCP
    /// field names and checkbox captions derive from it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Taxi => "taxi",
            Self::Bus => "bus",
            Self::Metro => "metro",
            Self::MetroOverlay => "metro overlay",
            Self::Underground => "underground",
            Self::Batobus => "batobus",
            Self::Water => "water",
            Self::Landmarks => "landmarks",
            Self::Stations => "stations",
            Self::Labels => "labels",
            Self::Numbers => "numbers",
            Self::Scores => "scores",
            Self::Compass => "compass",
            Self::Bridges => "bridges",
        }
    }
}

impl fmt::Display for LayerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
