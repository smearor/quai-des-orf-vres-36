//! `Street` - a decorative street drawn inside a block.

use crate::Polyline;
use crate::Segment;
use crate::map::Polygon;
use crate::map::blocks::Angle;
use crate::map::blocks::SquareMillimeters;
use crate::map::blocks::Steps;
use crate::map::blocks::decoration::Streets;
use std::f64::consts::PI;
use std::ops::Deref;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Smallest block area that gets a decorative street (mm²).
const STREET_MIN_AREA: SquareMillimeters = SquareMillimeters::new(1100.0);
/// Area that earns a second street (mm²).
const STREET_HUGE_AREA: SquareMillimeters = SquareMillimeters::new(2800.0);
/// Clearance a street chord keeps at its ends (mm).
const STREET_END_GAP: Millimeters = Millimeters::new(2.0);
/// Minimum chord span (`4 * STREET_END_GAP`): after the
/// `STREET_END_GAP` trim at both ends at least twice the end
/// gap of street stays visible.
const STREET_MIN_SPAN: Millimeters = Millimeters::new(8.0);
/// Candidate street directions (evenly spread over 180°).
const STREET_DIRS: usize = 10;

/// Bend subdivisions along a decorative street.
const BEND_STEPS: Steps = Steps::new(5);

/// A decorative street inside a block: a gently curved polyline
/// the houses keep clear of (a corridor obstacle), not a block
/// boundary.
#[derive(Debug, Clone)]
pub struct Street(pub Polyline);

impl Street {
    /// The street's end-to-end chord: first to last waypoint -
    /// the direction a second street must not parallel.
    pub fn segment(&self) -> Option<Segment<Millimeters>> {
        Some(Segment::new(*self.first()?, *self.last()?))
    }

    /// Decorative streets inside `outline`, map mm: gently curved
    /// paths through (or beside) the block's deepest point,
    /// clipped to the outline and shortened at their ends.
    /// Streets in the same block never cross; empty for blocks
    /// too small to host one.
    pub fn of_block(outline: &Polygon, seed: u64) -> Streets {
        let area = outline.signed_area();
        if area < STREET_MIN_AREA {
            return Streets::default();
        }
        let Some(anchor) = outline.deepest_point() else { return Streets::default() };
        let count = if area > STREET_HUGE_AREA { 2 } else { 1 };
        // Rank the seeded candidate directions by clipped chord length.
        let base = seed_angle(seed);
        let mut chords: Vec<(Millimeters, Segment<Millimeters>)> = (0..STREET_DIRS)
            .filter_map(|i| {
                let angle = Angle::new(base + PI * i as f64 / STREET_DIRS as f64);
                let chord = chord_through(outline, &anchor, angle)?;
                Some((chord.from.distance(&chord.to), chord))
            })
            .collect();
        chords.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut streets = Streets::default();
        for (index, (_, chord)) in chords.iter().enumerate() {
            if streets.len() >= count {
                break;
            }
            // A second street runs in another direction, anchored
            // off the first street's line so the two never cross.
            let direction = chord.direction();
            if streets.has_parallel(direction) {
                continue;
            }
            let street_anchor = match streets.first().and_then(|first| first.segment()) {
                None => anchor,
                Some(chord) => {
                    let axis = chord.to - chord.from;
                    let length = axis.magnitude().max(Millimeters::new(1e-9));
                    let shift = (length.as_f64() * 0.28).min(14.0) * if noise(seed, index as u64, 6) < 0.5 { 1.0 } else { -1.0 };
                    anchor + axis.normalized().perpendicular() * shift
                }
            };
            let Some(straight) = chord_through(outline, &street_anchor, direction) else {
                continue;
            };
            let candidate = Street::bend(outline, &straight, seed, index as u64);
            if !streets.has_crossing(&candidate) {
                streets.push(candidate);
            }
        }
        streets
    }

    /// A street bent into a gentle single-arc curve: the
    /// endpoints stay put, interior points slide perpendicular
    /// by a seeded sine amplitude. Falls back to progressively
    /// flatter bends until the whole curve stays inside the
    /// outline (the straight chord always does).
    pub fn bend(outline: &Polygon, chord: &Segment<Millimeters>, seed: u64, slot: u64) -> Street {
        let side = if noise(seed, slot, 7) < 0.5 { 1.0 } else { -1.0 };
        let mut amplitude = chord.from.distance(&chord.to) * (0.05 + 0.09 * noise(seed, slot, 8)) * side;
        for _ in 0..4 {
            let curve = chord.arc(BEND_STEPS.as_usize(), amplitude);
            if curve.iter().all(|point| outline.contains(point)) {
                return Street(curve);
            }
            amplitude = amplitude * 0.5;
        }
        Street(Polyline::new(vec![chord.from, chord.to]))
    }
}

impl Deref for Street {
    type Target = Polyline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The chord of `outline` through `anchor` along `angle`: the two
/// outline crossings bracketing the anchor, shortened by
/// `STREET_END_GAP` at each end.
fn chord_through(outline: &Polygon, anchor: &Vec2<Millimeters>, angle: Angle) -> Option<Segment<Millimeters>> {
    let axis = Vec2::new(angle.as_f64().cos(), angle.as_f64().sin());
    let chord = outline.chord(anchor, &axis)?;
    let direction = chord.to - chord.from;
    let length = direction.magnitude();
    if length < STREET_MIN_SPAN {
        return None;
    }
    let unit = direction / length.as_f64();
    Some(Segment::new(chord.from + unit * STREET_END_GAP.as_f64(), chord.to - unit * STREET_END_GAP.as_f64()))
}

/// Deterministic street direction seed.
fn seed_angle(seed: u64) -> f64 {
    noise(seed, 0, 5) * PI
}

/// Deterministic hash noise in `0.0..1.0`.
fn noise(seed: u64, slot: u64, channel: u64) -> f64 {
    crate::map::blocks::noise::noise(seed, 0, 0, slot as usize, channel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big_square() -> Polygon {
        Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(60.0, 0.0), Vec2::new(60.0, 60.0), Vec2::new(0.0, 60.0)])
    }

    #[test]
    fn a_big_block_gets_streets() {
        let outline = big_square();
        let streets = Street::of_block(&outline, 1);
        assert!(!streets.is_empty());
        for chord in &streets {
            for point in chord.iter() {
                assert!(outline.contains(point), "{point:?}");
            }
            assert!(chord.first().unwrap().distance(chord.last().unwrap()) > Millimeters::new(20.0));
        }
    }

    #[test]
    fn a_small_block_gets_no_street() {
        let outline = Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(20.0, 0.0), Vec2::new(20.0, 20.0), Vec2::new(0.0, 20.0)]);
        assert!(Street::of_block(&outline, 1).is_empty());
    }
}
