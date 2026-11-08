//! `Placement` - the state shared by the row and corner passes.

use crate::map::Polygon;
use crate::map::blocks::House;
use crate::map::blocks::Obstacles;

/// Shared placement state for one ring's passes: the block
/// outline every footprint must stay inside, the obstacles
/// carving the block (tunnels, decorative streets, parks), the
/// houses placed so far and the determinism channels.
pub(super) struct Placement<'a> {
    /// Block outline all footprints must stay inside.
    pub(super) outline: &'a Polygon,
    /// Obstacle polygons inside the block - houses in their way
    /// are skipped.
    pub(super) obstacles: &'a Obstacles,
    /// Houses placed so far (row and corner passes append).
    pub(super) placed: Vec<House>,
    /// Determinism seed for the noise channels.
    pub(super) seed: u64,
    /// Ring index mixed into the noise channels.
    pub(super) ring_index: usize,
}

impl Placement<'_> {
    /// Whether a candidate footprint may be placed: its interior
    /// points (from `inside_from` on) stay inside the block
    /// `outline` (facade points may ride on it), it overlaps no
    /// obstacle (tunnels, decorative streets, parks) and no
    /// already-placed house.
    pub(super) fn fits(&self, footprint: &Polygon, inside_from: usize) -> bool {
        footprint[inside_from..].iter().all(|p| self.outline.contains_or_touches(p))
            && !self.obstacles.iter().any(|obstacle| footprint.overlaps(obstacle))
            && !self.placed.iter().any(|other| footprint.overlaps(&other.footprint))
    }
}
