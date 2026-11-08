//! Parks: named green areas from the map dataset.

//! `Park` - a named green spot from the map dataset.
//!
//! Large blocks may carry a park: the dataset pins it by name and
//! position so the green stays geographically close to reality
//! (Jardin du Luxembourg, Buttes-Chaumont, ...). The block keeps
//! the park free of houses and the renderer draws a green blob
//! with the park's name.

pub mod name;

pub use name::ParkName;

use crate::map::Polygon;
use crate::map::blocks::Steps;
use serde::Deserialize;
use serde::Serialize;
use std::f64::consts::TAU;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Smallest park clearance worth drawing (mm).
const PARK_MIN_RADIUS: Millimeters = Millimeters::new(4.5);
/// Largest park blob radius (mm).
const PARK_MAX_RADIUS: Millimeters = Millimeters::new(11.0);
/// Clearance a park keeps to the block outline (mm).
const PARK_GAP: Millimeters = Millimeters::new(2.0);
/// Blob polygon vertices.
const PARK_STEPS: Steps = Steps::new(9);

/// A named park marker the owning block keeps free of houses.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Park {
    /// Park name (proper noun, not localized).
    pub name: ParkName,
    /// Park center in map millimeters; TOML shape `pos_mm = [x, y]`.
    #[serde(rename = "pos_mm", with = "tangible_objects_model::serde_pair")]
    pub pos: Vec2<Millimeters>,
}

impl Park {
    /// The organic green blob the park grows inside `outline`:
    /// `PARK_STEPS` vertices with seeded radial noise. `None`
    /// when the park's position is outside the outline or the
    /// available clearance is too small - the block then keeps
    /// its houses instead.
    pub fn polygon(&self, outline: &Polygon, seed: u64) -> Option<Polygon> {
        let pos = &self.pos;
        if !outline.contains(pos) {
            return None;
        }
        let clearance = outline.clearance(pos);
        let radius = (clearance - PARK_GAP).min(PARK_MAX_RADIUS);
        if radius < PARK_MIN_RADIUS {
            return None;
        }
        let mut blob: Vec<Vec2<Millimeters>> = (0..PARK_STEPS.as_usize())
            .map(|k| {
                let angle = TAU * k as f64 / PARK_STEPS.as_f64();
                let r = radius.as_f64() * (0.82 + 0.33 * noise(seed, k as u64, 7));
                Vec2::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r)
            })
            .collect();
        // A lobe poking out of the block shrinks toward the center.
        for _ in 0..3 {
            if blob.iter().all(|p| outline.contains(p)) {
                break;
            }
            blob = blob
                .iter()
                .map(|p| Vec2::new(pos.x + (p.x - pos.x) * 0.8, pos.y + (p.y - pos.y) * 0.8))
                .collect();
        }
        blob.iter().all(|p| outline.contains(p)).then_some(Polygon::new(blob))
    }
}

/// Deterministic hash noise in `0.0..1.0`.
fn noise(seed: u64, slot: u64, channel: u64) -> f64 {
    crate::map::blocks::noise::noise(seed, 0, 0, slot as usize, channel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big_square() -> Polygon {
        Polygon::new(vec![Vec2::new(10.0, 10.0), Vec2::new(60.0, 10.0), Vec2::new(60.0, 60.0), Vec2::new(10.0, 60.0)])
    }

    #[test]
    fn a_park_blob_fits_its_block() {
        let outline = big_square();
        let pos = Vec2::new(30.0, 30.0);
        let park = Park {
            name: ParkName::from("test"),
            pos,
        };
        let blob = park.polygon(&outline, 1).expect("park");
        assert!(blob.len() == PARK_STEPS.as_usize());
        for point in &blob {
            assert!(outline.contains(point));
        }
        assert!(blob.iter().any(|p| p.distance(&pos) > PARK_MIN_RADIUS));
    }

    #[test]
    fn a_park_too_close_to_the_edge_yields_none() {
        let outline = big_square();
        let near = Park {
            name: ParkName::from("test"),
            pos: Vec2::new(59.5, 30.0),
        };
        let outside = Park {
            name: ParkName::from("test"),
            pos: Vec2::new(70.0, 30.0),
        };
        assert!(near.polygon(&outline, 1).is_none());
        assert!(outside.polygon(&outline, 1).is_none());
    }
}
