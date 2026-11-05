//! `LaneConstraint` - a lane-continuity vote between two hops.

use qdo36_map::StationPair;

/// A lane-continuity constraint between two hops sharing a vertex:
/// `relation` is the sign `side(to)` should take relative to
/// `side(from)` so the lane keeps its spatial side through the
/// vertex (`+1` same side, `-1` mirrored hop ordering).
pub struct LaneConstraint {
    /// The hop the edge arrives on.
    pub from: StationPair,
    /// The hop the edge departs on.
    pub to: StationPair,
    /// The desired side relation: `side(to) = side(from) * relation`.
    pub relation: f64,
}
