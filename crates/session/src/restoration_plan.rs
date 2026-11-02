//! `RestorationPlan` - the physical steps needed to reach a previous
//! board state after `undo`/`redo`.

use serde::Deserialize;
use serde::Serialize;

use crate::GameState;
use crate::RestoreStep;

/// The list of physical pawn corrections between two game states.
///
/// The plan deliberately only covers detective pawns: Fugitive has no
/// physical token, so undoing a Fugitive move yields an empty plan -
/// the secrecy invariant survives undo (WP06 R8b).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestorationPlan {
    /// Pawn relocations in seat order; empty when nothing physical
    /// changes.
    pub steps: Vec<RestoreStep>,
}

impl RestorationPlan {
    /// The plan that physically turns `after` back into `before`.
    pub fn between(after: &GameState, before: &GameState) -> Self {
        let steps = after
            .positions
            .iter()
            .filter_map(|(pawn, now)| {
                let then = before.positions.get(pawn).copied();
                (then != Some(*now))
                    .then(|| then.map(|from| RestoreStep { pawn: *pawn, from, to: *now }))
                    .flatten()
            })
            .collect();
        Self { steps }
    }
}
