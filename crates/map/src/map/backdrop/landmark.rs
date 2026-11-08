use crate::map::Polygon;
use crate::map::backdrop::LandmarkKind;
use crate::map::backdrop::LandmarkName;
use crate::map::blocks::Steps;
use serde::Deserialize;
use serde::Serialize;
use std::f64::consts::TAU;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;
use tangible_objects_model::serde_pair;

/// A decorative landmark marker on the map (Tour Eiffel,
/// Fernsehturm, Brandenburger Tor, ...).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Landmark {
    /// Display name (proper noun, not localized).
    pub name: LandmarkName,
    /// Position in map millimeters; TOML shape `pos_mm = [x, y]`.
    #[serde(rename = "pos_mm", with = "serde_pair")]
    pub pos: Vec2<Millimeters>,
    /// What kind of landmark this is (drives the icon).
    pub kind: LandmarkKind,
}

impl Landmark {
    /// The landmark's building footprint inside `outline`: a
    /// regular octagon around its position, shrinking until it
    /// fits; `None` when the block has no room - the landmark
    /// stays a pin.
    pub fn footprint(&self, outline: &Polygon) -> Option<Polygon> {
        let pos = &self.pos;
        if !outline.contains(pos) {
            return None;
        }
        LANDMARK_RADII.iter().copied().find_map(|radius| {
            let poly: Vec<Vec2<Millimeters>> = (0..LANDMARK_STEPS.as_usize())
                .map(|k| {
                    let angle = TAU * k as f64 / LANDMARK_STEPS.as_f64() + TAU / 16.0;
                    Vec2::new(pos.x + angle.cos() * radius.as_f64(), pos.y + angle.sin() * radius.as_f64())
                })
                .collect();
            poly.iter().all(|p| outline.contains(p)).then_some(Polygon::new(poly))
        })
    }
}

/// Radii a landmark volume tries inside its block (mm, largest
/// first).
const LANDMARK_RADII: [Millimeters; 4] = [Millimeters::new(5.5), Millimeters::new(4.2), Millimeters::new(3.2), Millimeters::new(2.4)];
/// Octagon vertices of a landmark footprint.
const LANDMARK_STEPS: Steps = Steps::new(8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_landmark_footprint_fits_its_block() {
        let outline = Polygon::new(vec![Vec2::new(0.0, 0.0), Vec2::new(60.0, 0.0), Vec2::new(60.0, 60.0), Vec2::new(0.0, 60.0)]);
        let landmark = Landmark {
            name: LandmarkName::from("test"),
            pos: Vec2::new(30.0, 30.0),
            kind: LandmarkKind::Monument,
        };
        let footprint = landmark.footprint(&outline).expect("footprint");
        assert_eq!(footprint.len(), LANDMARK_STEPS.as_usize());
        for point in &footprint {
            assert!(outline.contains(point));
        }
        // No room near the edge or outside the block.
        let near = Landmark {
            pos: Vec2::new(59.0, 30.0),
            ..landmark.clone()
        };
        let outside = Landmark {
            pos: Vec2::new(70.0, 30.0),
            ..landmark
        };
        assert!(near.footprint(&outline).is_none());
        assert!(outside.footprint(&outline).is_none());
    }
}
