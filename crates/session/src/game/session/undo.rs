//! Undo/redo: chess-style memento over the `past`/`future` stacks,
//! returning a physical [`RestorationPlan`] per step.

use crate::RestorationPlan;
use crate::game::session::GameSession;

impl GameSession {
    /// Whether undo is possible right now (also gated by
    /// `config.allow_undo`).
    pub fn can_undo(&self) -> bool {
        self.config.allow_undo && !self.past.is_empty()
    }

    /// Whether redo is possible right now.
    pub fn can_redo(&self) -> bool {
        self.config.allow_undo && !self.future.is_empty()
    }

    /// Reverts the last move; the returned plan lists the physical
    /// pawn relocations (empty for Fugitive - nothing on the board,
    /// secrecy preserved).
    pub fn undo(&mut self) -> Option<RestorationPlan> {
        if !self.can_undo() {
            return None;
        }
        let previous = self.past.pop()?;
        let plan = RestorationPlan::between(&self.state, &previous);
        self.future.push(std::mem::replace(&mut self.state, previous));
        Some(plan)
    }

    /// Re-applies the last undone move.
    pub fn redo(&mut self) -> Option<RestorationPlan> {
        if !self.can_redo() {
            return None;
        }
        let next = self.future.pop()?;
        let plan = RestorationPlan::between(&self.state, &next);
        self.past.push(std::mem::replace(&mut self.state, next));
        Some(plan)
    }
}
