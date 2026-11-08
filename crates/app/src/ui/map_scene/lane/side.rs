//! `LaneSide` - the side of a hop a lane rides on.

use crate::ui::map_scene::lane::SideRelation;
use std::ops::Mul;

/// The side of a hop a lane rides on (`Left` is canonical
/// `+1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneSide {
    /// Left of the hop's direction.
    Left,
    /// Right of the hop's direction.
    Right,
}

impl LaneSide {
    /// The mirrored side.
    pub fn flipped(&self) -> LaneSide {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// The side as a ±1 factor for offset multiplication.
    pub fn factor(&self) -> f64 {
        match self {
            Self::Left => 1.0,
            Self::Right => -1.0,
        }
    }
}

impl Mul<SideRelation> for LaneSide {
    type Output = LaneSide;

    /// The side carried through a vertex: `Same` keeps it,
    /// `Flip` mirrors it.
    fn mul(self, relation: SideRelation) -> LaneSide {
        match relation {
            SideRelation::Same => self,
            SideRelation::Flip => self.flipped(),
        }
    }
}
