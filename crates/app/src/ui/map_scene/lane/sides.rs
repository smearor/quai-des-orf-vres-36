//! `LaneSides` - the resolved lane side per hop.

use crate::ui::map_scene::lane::LaneConstraint;
use crate::ui::map_scene::lane::LaneSide;
use qdo36_map::StationPair;
use std::collections::HashMap;
use std::ops::Deref;

/// The lane side each `StationPair` resolves to, propagated
/// through the vertex constraints - Derefs to the map.
#[derive(Debug, Clone, Default)]
pub struct LaneSides(pub HashMap<StationPair, LaneSide>);

impl LaneSides {
    /// The sides `constraints` resolve the hops to - each
    /// unconstrained vertex seeds `Left`, two passes propagate
    /// the relations through.
    pub fn solve(constraints: &[LaneConstraint]) -> LaneSides {
        let mut sides = LaneSides::default();
        for _ in 0..2 {
            for constraint in constraints {
                let (from, to, relation) = (constraint.from, constraint.to, constraint.relation);
                match (sides.get(&from).copied(), sides.get(&to).copied()) {
                    (Some(s_from), None) => {
                        sides.0.insert(to, s_from * relation);
                    }
                    (None, Some(s_to)) => {
                        sides.0.insert(from, s_to * relation);
                    }
                    (None, None) => {
                        sides.0.insert(from, LaneSide::Left);
                        sides.0.insert(to, LaneSide::Left * relation);
                    }
                    _ => {}
                }
            }
        }
        sides
    }

    /// The hop's lane side - `Left` when unconstrained.
    pub fn of(&self, hop: &StationPair) -> LaneSide {
        self.0.get(hop).copied().unwrap_or(LaneSide::Left)
    }
}

impl Deref for LaneSides {
    type Target = HashMap<StationPair, LaneSide>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
