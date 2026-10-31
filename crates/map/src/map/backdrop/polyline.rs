use serde::Deserialize;
use serde::Serialize;
use serde::de::Error as _;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// A connected sequence of map-mm points (river course, street).
///
/// Serializes flat like upstream `flat_points`: `[x0, y0, x1, y1, ...]`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Polyline(pub Vec<Vec2<Millimeters>>);

impl Polyline {
    /// The polyline's points.
    pub fn points(&self) -> &[Vec2<Millimeters>] {
        &self.0
    }
}

impl Serialize for Polyline {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut flat = Vec::with_capacity(self.0.len() * 2);
        for p in &self.0 {
            flat.push(p.x);
            flat.push(p.y);
        }
        flat.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Polyline {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let flat = <Vec<f64>>::deserialize(deserializer)?;
        if flat.len() % 2 != 0 {
            return Err(D::Error::custom("polyline must contain an even number of coordinates"));
        }
        let (pairs, _rest) = flat.as_chunks::<2>();
        let points = pairs.iter().map(|xy| Vec2::new(xy[0], xy[1])).collect();
        Ok(Self(points))
    }
}
