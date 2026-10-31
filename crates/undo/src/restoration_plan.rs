use crate::RestoreStep;
use serde::Deserialize;
use serde::Serialize;

/// The physical steps that restore a previous board position after an
/// undo/redo.
///
/// Empty for Corbeau-only moves - his position is hidden and has no
/// physical marker (secrecy invariant). Each step is guided by a ghost
/// highlight until the placement gate confirms it (WP07).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestorationPlan {
    /// Physical steps in execution order.
    pub steps: Vec<RestoreStep>,
}

impl RestorationPlan {
    /// Whether the plan needs no physical correction.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}
