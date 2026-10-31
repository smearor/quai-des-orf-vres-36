//! Physical undo for Quai des Orfèvres 36.
//!
//! [`RestorationPlan`]/[`RestoreStep`] - the physical marker moves that
//! restore a previous board position after undo/redo. Fugitive undo
//! produces an empty plan: his hidden position has no marker.

pub mod restoration_plan;
pub mod restore_step;

pub use restoration_plan::RestorationPlan;
pub use restore_step::RestoreStep;
