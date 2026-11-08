//! `BlockCut` - which feature one side of a city block comes from.

use crate::StationPair;

/// The feature a block boundary edge comes from - drives the
/// setback a rendered block keeps from it. `Ord` is the margin
/// preference when coinciding fragments compete: a street
/// attribute wins over water, water over the map border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockCut {
    /// A street hop between two stations (taxi/bus edge geometry).
    Street(StationPair),
    /// A piece of a water course (the river band's centerline).
    Water,
    /// A piece of the map border (the bounds rectangle).
    Border,
}
