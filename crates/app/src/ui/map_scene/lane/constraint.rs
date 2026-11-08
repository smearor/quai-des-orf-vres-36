//! `LaneConstraint` - a lane-continuity vote between two hops.

use crate::ui::map_scene::lane::SideRelation;
use qdo36_map::StationPair;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

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
    pub relation: SideRelation,
}

impl LaneConstraint {
    /// The constraints along `hops`' junctions given each hop's
    /// lane `rank` and `normal_of`: `relation` is the sign of
    /// `rank_u * rank_v * dot(n_u, n_v)` - junctions whose
    /// normals are missing or whose product vanishes contribute
    /// nothing.
    pub fn chain(hops: &[StationPair], ranks: &[f64], normal_of: impl Fn(StationPair) -> Option<Vec2<Millimeters>>) -> Vec<LaneConstraint> {
        let mut constraints = Vec::new();
        for i in 1..hops.len() {
            let (from, to) = (hops[i - 1], hops[i]);
            let (Some(n_from), Some(n_to)) = (normal_of(from), normal_of(to)) else {
                continue;
            };
            let relation = ranks[i - 1] * ranks[i] * n_from.dot::<f64>(&n_to);
            if relation.abs() > 1e-9 {
                constraints.push(Self {
                    from,
                    to,
                    relation: SideRelation::of(relation),
                });
            }
        }
        constraints
    }
}
