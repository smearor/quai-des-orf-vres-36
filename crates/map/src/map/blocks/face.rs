//! `BlockFace` - one free area enclosed by streets, river and border.

use crate::Segment;
use crate::map::DistrictName;
use crate::map::Park;
use crate::map::Polygon;
use crate::map::backdrop::Landmark;
use crate::map::blocks::BlockCut;
use crate::map::blocks::SquareMillimeters;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

/// The smallest face area worth drawing (mm²) - slivers from
/// snapped T-junctions and dead ends drop below it.
const MIN_AREA: SquareMillimeters = SquareMillimeters::new(4.0);

/// A free area of the map enclosed by streets, river courses and
/// the map border - the footprint a city block is drawn into.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockFace {
    /// Outline in map mm with positive signed area (the interior
    /// lies left of each edge's direction).
    pub outline: Polygon,
    /// What cuts each outline edge: `cuts[i]` is the feature
    /// bounding `outline[i] -> outline[i+1]` (same length as
    /// `outline`).
    pub cuts: Vec<BlockCut>,
    /// Underground hop segments crossing this face - only the
    /// houses on a tunnel's way stay out; the block itself is
    /// still drawn.
    pub tunnels: Vec<Segment<Millimeters>>,
    /// The district name this block answers to, for labels and
    /// later game communication; `None` for outer faces and maps
    /// whose dataset names no quarters.
    pub district: Option<DistrictName>,
    /// Named parks inside this face - large blocks keep them free
    /// of houses and draw a green spot.
    pub parks: Vec<Park>,
    /// Backdrop landmarks sitting inside this face - they render
    /// as landmark volumes among the houses.
    pub landmarks: Vec<Landmark>,
}

impl BlockFace {
    /// The face's signed area in mm² (positive by construction).
    pub fn signed_area(&self) -> SquareMillimeters {
        self.outline.signed_area()
    }

    /// Whether the face touches the map border - those outer faces
    /// form the countryside ring around the city and carry no
    /// blocks (a texture fills them instead).
    pub fn is_outer(&self) -> bool {
        self.cuts.contains(&BlockCut::Border)
    }

    /// Whether a hop between two positions touches the face's
    /// interior - strict edge crossings plus five samples along the
    /// segment (endpoints sitting on the boundary alone do not
    /// count as touching).
    pub fn touched_by(&self, segment: &Segment<Millimeters>) -> bool {
        let n = self.outline.len();
        if (0..n).any(|i| segment.crosses(&Segment::new(self.outline[i], self.outline[(i + 1) % n]))) {
            return true;
        }
        (1..5).any(|k| self.contains(&(segment.from + (segment.to - segment.from) * (k as f64 / 5.0))))
    }

    /// Records `segment` as a tunnel crossing this face when it
    /// touches the interior (see [`Self::touched_by`]).
    pub fn attach_tunnel(&mut self, segment: Segment<Millimeters>) {
        if self.touched_by(&segment) {
            self.tunnels.push(segment);
        }
    }

    /// A point inside the face with the widest clearance to the
    /// boundary - where a district label sits best.
    pub fn label_anchor(&self) -> Option<Vec2<Millimeters>> {
        self.outline.deepest_point()
    }

    /// The block outline pulled `margin(cut)` mm inward from each
    /// bounding feature - the street corridor, river band or border
    /// clearance the drawn block keeps. Corner vertices are the
    /// intersections of the two offset edge lines, miter-capped.
    /// `None` when the face cannot take the margins (the inset
    /// collapses or leaves the face) - after one retry at half
    /// margins the face is left out entirely.
    pub fn inset(&self, margin: &dyn Fn(&BlockCut) -> f64) -> Option<Polygon> {
        for scale in [1.0, 0.5] {
            let outline = self.offset(scale, margin);
            let valid = outline.len() >= 3 && outline.signed_area() > MIN_AREA && outline.iter().all(|point| self.contains(point)) && outline.is_simple();
            if valid {
                return Some(outline);
            }
        }
        None
    }

    /// Whether `point` lies strictly inside the face (ray casting
    /// over the outline edges).
    pub fn contains(&self, point: &Vec2<Millimeters>) -> bool {
        self.outline.contains(point)
    }

    /// The face outline pushed `margin(cut)` mm outward along each
    /// edge - the ground fill grows to the neighboring faces'
    /// inset edges this way, so streets and corridors always rest
    /// on city ground.
    pub fn outset(&self, margin: &dyn Fn(&BlockCut) -> f64) -> Polygon {
        let distances: Vec<f64> = self.cuts.iter().map(|cut| -margin(cut)).collect();
        self.outline.offset(&distances)
    }

    /// One inset attempt at `scale` times the per-edge margins.
    fn offset(&self, scale: f64, margin: &dyn Fn(&BlockCut) -> f64) -> Polygon {
        let distances: Vec<f64> = self.cuts.iter().map(|cut| margin(cut) * scale).collect();
        self.outline.offset(&distances)
    }

    /// Removes dead-end excursions from the traced face: where a
    /// vertex's predecessor and successor coincide
    /// (`... A, B, A, ...`), the two middle entries collapse and
    /// the boundary continues through `A`.
    pub(super) fn remove_spikes(&mut self) {
        loop {
            let n = self.outline.len();
            if n < 3 {
                return;
            }
            let Some(i) = (0..n).find(|&i| self.outline[(i + n - 1) % n] == self.outline[(i + 1) % n]) else {
                return;
            };
            // The surviving predecessor edge takes over the cut of
            // the removed successor edge.
            self.cuts[(i + n - 1) % n] = self.cuts[(i + 1) % n];
            self.outline.remove(i);
            self.cuts.remove(i);
            let next = i % self.outline.len();
            self.outline.remove(next);
            self.cuts.remove(next);
        }
    }
}

/// Whether `point` lies strictly inside `polygon` (ray casting over
/// the polygon edges).
/// A point inside `points` with the widest clearance to the
/// boundary - where a district label sits best. Scans a coarse
/// grid over the bounding box and keeps the deepest inside point;
/// `None` when the polygon holds no grid point at all.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::StationId;
    use crate::StationPair;

    fn square_face() -> BlockFace {
        let stations = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)];
        BlockFace {
            outline: Polygon::new(stations.to_vec()),
            cuts: vec![
                BlockCut::Street(StationPair::new(StationId::new(1), StationId::new(2))),
                BlockCut::Street(StationPair::new(StationId::new(2), StationId::new(3))),
                BlockCut::Water,
                BlockCut::Border,
            ],
            tunnels: Vec::new(),
            district: None,
            parks: Vec::new(),
            landmarks: Vec::new(),
        }
    }

    #[test]
    fn inset_shrinks_each_edge_by_its_margin() {
        let face = square_face();
        let margin = |cut: &BlockCut| match cut {
            BlockCut::Street(_) => 1.0,
            BlockCut::Water => 2.0,
            BlockCut::Border => 0.5,
        };
        let inset = face.inset(&margin).unwrap_or_else(|| panic!("inset"));
        assert_eq!(inset.len(), 4);
        // Left edge (cut[3], border) moves 0.5 inward: x = 0.5.
        // Bottom edge (cut[0], street) moves 1.0: y = 1.0.
        // Right edge (cut[1], street) moves 1.0: x = 9.0.
        // Top edge (cut[2], water) moves 2.0: y = 8.0.
        assert!(inset.iter().all(|p| (0.4..=9.1).contains(&p.x) && (0.9..=8.1).contains(&p.y)), "{inset:?}");
        assert!(inset.iter().any(|p| (p.x - 0.5).abs() < 1e-6 && (p.y - 1.0).abs() < 1e-6), "{inset:?}");
    }

    #[test]
    fn inset_drops_a_face_too_small_for_the_margins() {
        let mut face = square_face();
        face.outline = Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0), Vec2::new(0.0, 4.0)]);
        assert!(face.inset(&|_| 3.0).is_none());
    }

    #[test]
    fn contains_works() {
        let face = square_face();
        assert!(face.contains(&Vec2::new(5.0, 5.0)));
        assert!(!face.contains(&Vec2::new(15.0, 5.0)));
    }
}
