//! `buildings` - procedural house footprints inside a block face.
//!
//! Houses line the block's inset outline like a perimeter row: each
//! house fronts its street, rolls its own depth (0.5 mm steps
//! inside `min_depth..=max_depth`) and keeps a gap to its
//! neighbors. Frontages vary between 1 and 4 units; sharp corners
//! get a flatiron house whose rounded nose points at the corner.
//! Underground tunnel segments cut the row: only the houses in
//! their way drop out, the rest of the row is still drawn. A face
//! too small for any row house becomes one block-filling house.
//! Placement is deterministic per seed - the same face always
//! grows the same houses.

use crate::map::Polygon;
use crate::map::blocks::Angle;
use crate::map::blocks::Depth;
use crate::map::blocks::Height;
use crate::map::blocks::House;
use crate::map::blocks::Obstacles;
use crate::map::blocks::SquareMillimeters;
use crate::map::blocks::Steps;
use crate::map::blocks::Units;
use crate::map::blocks::Variant;
use crate::map::blocks::noise::noise;

use smearor_commons_geometry::Millimeters;
use std::f64::consts::TAU;

mod corner_footprint;
mod corner_house;
mod corner_nose;
mod corner_wedge;
mod edge_frame;
mod placement;

use corner_footprint::CornerFootprint;
use corner_house::CornerHouse;
use corner_nose::CornerNose;
use corner_wedge::CornerWedge;
use edge_frame::EdgeFrame;
use placement::Placement;

/// Interior angle below which a vertex counts as a sharp corner
/// and gets a rounded flatiron house (~140°).
const CORNER_MAX_ANGLE: Angle = Angle::new(2.6);

/// Arc segments approximating the flatiron nose.
const CORNER_ARC_STEPS: Steps = Steps::new(7);

/// Sizes for the procedural houses inside a block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingParams {
    /// Shallowest house depth.
    pub min_depth: Depth,
    /// Deepest house depth; each house rolls its own depth in
    /// 0.5 mm steps inside `min_depth..=max_depth`.
    pub max_depth: Depth,
    /// Front width of one unit; houses are 1-4 units wide.
    pub width: Millimeters,
    /// Gap between neighboring houses.
    pub gap: Millimeters,
    /// Free space between two house rings - only used when a
    /// future block grows a second ring.
    pub ring_gap: Millimeters,
    /// Space kept free at block corners so the corner houses of
    /// two adjacent streets never collide.
    pub corner_clearance: Millimeters,
    /// Remaining interior area below which no inner ring is placed.
    pub min_ring_area: SquareMillimeters,
    /// How many concentric house rings a block gets at most.
    pub max_rings: usize,
}

impl Default for BuildingParams {
    fn default() -> Self {
        Self {
            min_depth: Depth::new(1.0),
            max_depth: Depth::new(3.0),
            width: Millimeters::new(4.0),
            gap: Millimeters::new(0.9),
            ring_gap: Millimeters::new(1.2),
            corner_clearance: Millimeters::new(1.6),
            min_ring_area: SquareMillimeters::new(60.0),
            max_rings: 1,
        }
    }
}

/// The signed arc sweep from `start` to `end` (radians) that passes
/// through `through` - picks the winding containing the interior
/// bisector.
fn arc_sweep(start: f64, end: f64, through: f64) -> f64 {
    let forward = (end - start).rem_euclid(TAU);
    if (through - start).rem_euclid(TAU) <= forward + 1e-9 {
        forward
    } else {
        forward - TAU
    }
}

impl BuildingParams {
    /// Fills `outline` (a block's inset outline, positive signed
    /// area) with procedural house footprints, deterministic per
    /// `seed`. `obstacles` are the convex polygons carving the
    /// block (tunnel corridors, decorative streets, parks) -
    /// houses in their way stay out.
    pub fn houses(&self, outline: &Polygon, seed: u64, obstacles: &Obstacles) -> Vec<House> {
        let mut placed: Vec<House> = Vec::new();
        let mut ring = outline.clone();
        for ring_index in 0..self.max_rings {
            let n = ring.len();
            if n < 3 {
                break;
            }
            let frames = EdgeFrame::of_ring(&ring);
            let mut ctx = Placement {
                outline,
                obstacles,
                placed,
                seed,
                ring_index,
            };
            // Space consumed at each edge's start/end by corner
            // houses (head = at edge start, tail = at edge end).
            let mut head = vec![self.corner_clearance; n];
            let mut tail = vec![self.corner_clearance; n];
            // Pass 1: flatiron houses on sharp vertices.
            for vertex in 0..n {
                if EdgeFrame::interior_angle(&frames, vertex) >= CORNER_MAX_ANGLE {
                    continue;
                }
                if let Some(corner) = self.corner_house(&ctx, &ring, &frames, vertex) {
                    tail[(vertex + n - 1) % n] = (corner.prev + self.gap).max(tail[(vertex + n - 1) % n]);
                    head[vertex] = (corner.next + self.gap).max(head[vertex]);
                    ctx.placed.push(corner.house);
                }
            }
            // Pass 2: row houses in the free space between corners.
            for edge in 0..n {
                self.row_houses(&mut ctx, &ring, &frames, edge, head[edge], tail[edge]);
            }
            placed = ctx.placed;
            if ring_index + 1 >= self.max_rings {
                break;
            }
            let distances = vec![self.max_depth.as_f64() + self.ring_gap.as_f64(); n];
            let next = ring.offset(&distances);
            if next.len() < 3 || next.signed_area() < self.min_ring_area || !next.iter().all(|p| outline.contains(p)) {
                break;
            }
            ring = next;
        }
        // A face too small for even one row house still gets a
        // building: the whole block footprint becomes the house
        // (a triangle then), unless an obstacle runs over it.
        if placed.is_empty() && !obstacles.iter().any(|obstacle| outline.overlaps(obstacle)) {
            placed.push(House {
                footprint: outline.clone(),
                units: Units::ONE,
                variant: Variant::of_seed(seed, 0, 0, 0),
                depth: self.max_depth,
                height: Height::roll(noise(seed, 0, 0, 0, 6)),
            });
        }
        placed
    }

    /// The depth a house rolls inside the params' range.
    fn depth_of(&self, roll: f64) -> Depth {
        Depth::roll(roll, self.min_depth, self.max_depth)
    }

    /// Row houses along edge `edge` of the ring, between `head`
    /// and `tail` clearance - widths roll 1-4 units, shrinking to
    /// fit.
    fn row_houses(&self, ctx: &mut Placement<'_>, ring: &Polygon, frames: &[EdgeFrame], edge: usize, head: Millimeters, tail: Millimeters) {
        let (start, frame) = (ring[edge], &frames[edge]);
        let end = frame.length - tail;
        let mut offset = head;
        let mut slot = 0usize;
        while offset + self.width <= end {
            let mut units = Units::roll(noise(ctx.seed, ctx.ring_index, edge, slot, 0));
            while units > Units::ONE && offset + self.width * units.as_f64() > end {
                units = units.saturating_dec();
            }
            let width = self.width * units.as_f64();
            let depth = self.depth_of(noise(ctx.seed, ctx.ring_index, edge, slot, 4));
            let depth_mm = depth.as_f64();
            let quad = Polygon::new(vec![
                start + frame.direction * offset.as_f64(),
                start + frame.direction * (offset + width).as_f64(),
                start + frame.direction * (offset + width).as_f64() + frame.inward * depth_mm,
                start + frame.direction * offset.as_f64() + frame.inward * depth_mm,
            ]);
            if ctx.fits(&quad, 2) {
                ctx.placed.push(House {
                    footprint: quad,
                    units,
                    variant: Variant::of_seed(ctx.seed, ctx.ring_index, edge, slot),
                    depth,
                    height: Height::roll(noise(ctx.seed, ctx.ring_index, edge, slot, 6)),
                });
            }
            offset = offset + width + self.gap;
            slot += 1;
        }
    }

    /// A flatiron house wrapping vertex `vertex` of the ring: the
    /// building fills the wedge along both edges, the street nose
    /// points at the vertex and is rounded - an arc tangent to
    /// both edges on the interior bisector. Flatirons are
    /// preferred and must measure at least 2 units wide and 2 mm
    /// deep; when an obstacle (a tunnel) or a narrow wedge cuts
    /// that build, a smaller variant is allowed. Returns the house
    /// plus the frontage consumed on the previous and next edge.
    fn corner_house(&self, ctx: &Placement<'_>, ring: &Polygon, frames: &[EdgeFrame], vertex: usize) -> Option<CornerHouse> {
        let units_roll = noise(ctx.seed, ctx.ring_index, vertex, 0, 3);
        let candidates = [
            // Preferred flatiron: at least 2 units, at least 2 mm.
            (
                if units_roll < 0.65 { Units::TWO } else { Units::new(3).unwrap_or(Units::TWO) },
                Depth::roll(noise(ctx.seed, ctx.ring_index, vertex, 0, 4), Depth::new(2.0), self.max_depth),
            ),
            // Relaxed fallback for cut or narrow corners.
            (
                if units_roll < 0.7 { Units::ONE } else { Units::TWO },
                Depth::roll(noise(ctx.seed, ctx.ring_index, vertex, 0, 5), self.min_depth, Depth::new(2.0)),
            ),
        ];
        for (units, depth) in candidates {
            let Some(corner) = self.corner_footprint(ring, frames, vertex, units, depth) else {
                continue;
            };
            // Interior points (from the nose tangent on) must stay
            // inside the block; the facade front rides on the outline.
            if ctx.fits(&corner.outline, 2) {
                return Some(CornerHouse {
                    house: House {
                        footprint: corner.outline,
                        units,
                        variant: Variant::of_seed(ctx.seed, ctx.ring_index, vertex, 0),
                        depth,
                        height: Height::roll(noise(ctx.seed, ctx.ring_index, vertex, 0, 6)),
                    },
                    prev: corner.prev,
                    next: corner.next,
                });
            }
        }
        None
    }

    /// The flatiron footprint at vertex `vertex` for `units` and
    /// `depth`: facade along both edges, tangent nose arc at the
    /// vertex, back wall at `depth`. `None` when the wedge
    /// geometry degenerates.
    fn corner_footprint(&self, ring: &Polygon, frames: &[EdgeFrame], vertex: usize, units: Units, depth: Depth) -> Option<CornerFootprint> {
        let count = ring.len();
        let previous_edge = (vertex + count - 1) % count;
        let wedge = CornerWedge {
            corner: ring[vertex],
            prev: &frames[previous_edge],
            next: &frames[vertex],
            frontage: units.as_f64() * self.width,
            depth,
        };
        let interior = EdgeFrame::interior_angle(frames, vertex).as_f64();
        let CornerNose {
            front_prev,
            front_next,
            points,
            facade_prev,
            facade_next,
        } = wedge.nose(interior);
        let mut footprint = vec![front_prev];
        footprint.extend_from_slice(&points);
        footprint.push(front_next);
        footprint.extend(wedge.back_wall(front_prev, front_next));
        Some(CornerFootprint {
            outline: Polygon::new(footprint),
            prev: facade_prev,
            next: facade_next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Segment;
    use smearor_commons_geometry::Vec2;

    fn square() -> Vec<Vec2<Millimeters>> {
        vec![Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0), Vec2::new(40.0, 40.0), Vec2::new(0.0, 40.0)]
    }

    #[test]
    fn houses_line_the_outline_and_stay_inside() {
        let outline = Polygon::new(square());
        let houses = BuildingParams::default().houses(&outline, 7, &Obstacles::default());
        assert!(!houses.is_empty());
        for house in &houses {
            assert!(house.footprint.len() >= 3);
            assert!((Units::MIN..=Units::MAX).contains(&house.units.as_u8()), "{}", house.units);
            assert!((1.0..=3.0).contains(&house.depth.as_f64()), "{}", house.depth);
            // Interior points inside; facade points may ride on
            // the outline edge.
            for corner in &house.footprint[2..] {
                assert!(outline.contains_or_touches(corner), "{corner:?}");
            }
        }
        // Deterministic per seed.
        assert_eq!(houses, BuildingParams::default().houses(&outline, 7, &Obstacles::default()));
        // Flatiron houses on all four sharp corners + rows.
        assert!(houses.iter().any(|h| h.footprint.len() > 4), "flatiron houses expected");
        assert!(houses.len() > 8, "{} houses", houses.len());
    }

    #[test]
    fn house_widths_vary() {
        let outline = Polygon::new(square());
        let houses = BuildingParams::default().houses(&outline, 3, &Obstacles::default());
        let distinct: std::collections::BTreeSet<Units> = houses.iter().map(|h| h.units).collect();
        assert!(distinct.len() > 1, "expected varied widths: {distinct:?}");
    }

    #[test]
    fn a_tunnel_cuts_the_row() {
        let outline = Polygon::new(square());
        // Horizontal tunnel corridor through the bottom row's
        // depth band: every house it crosses drops out, the rest
        // still draws.
        let tunnel = Segment::new(Vec2::new(2.0, 1.3), Vec2::new(38.0, 1.3)).corridor(Millimeters::new(0.8));
        let with_tunnel = BuildingParams::default().houses(&outline, 7, &Obstacles(vec![tunnel]));
        let without = BuildingParams::default().houses(&outline, 7, &Obstacles::default());
        assert!(with_tunnel.len() < without.len(), "{with_tunnel:?} vs {without:?}");
        assert!(!with_tunnel.is_empty());
    }

    #[test]
    fn flatirons_fill_the_corners_at_least_two_units() {
        let outline = Polygon::new(square());
        let houses = BuildingParams::default().houses(&outline, 7, &Obstacles::default());
        // The four square corners all carry a flatiron (footprint >
        // 4 points thanks to the nose arc) of at least 2 units.
        let flatirons: Vec<&House> = houses.iter().filter(|h| h.footprint.len() > 4).collect();
        for house in &flatirons {
            eprintln!(
                "flatiron {:?} units {}",
                house.footprint.iter().map(|p| (p.x.round() as i64, p.y.round() as i64)).collect::<Vec<_>>(),
                house.units
            );
        }
        assert!(flatirons.len() >= 4, "{} flatirons", flatirons.len());
        for house in houses.iter().filter(|h| h.footprint.len() > 4) {
            assert!(house.units >= Units::TWO, "flatiron units {}", house.units);
            assert!(house.depth >= Depth::new(2.0), "flatiron depth {}", house.depth);
        }
    }

    #[test]
    fn a_tiny_face_gets_a_house() {
        let outline = Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(3.0, 0.0), Vec2::new(0.0, 3.0)]);
        let houses = BuildingParams::default().houses(&outline, 3, &Obstacles::default());
        // A tiny face yields at least one house (a corner flatiron
        // or the whole-face fallback).
        assert!(!houses.is_empty());
        for house in &houses {
            for corner in house.footprint.iter() {
                assert!(outline.contains_or_touches(corner), "{corner:?}");
            }
        }
    }

    #[test]
    fn a_thin_block_gets_houses() {
        let outline = Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(30.0, 0.0), Vec2::new(30.0, 8.0), Vec2::new(0.0, 8.0)]);
        let houses = BuildingParams::default().houses(&outline, 3, &Obstacles::default());
        for house in &houses {
            assert!(house.footprint.iter().all(|p| (-1.0..=31.0).contains(&p.x) && (-1.0..=9.0).contains(&p.y)));
        }
        assert!(!houses.is_empty());
    }
}
