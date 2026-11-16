//! `SegmentExt` - millimeter-specific segment derivations on top
//! of the generic `smearor_commons_geometry::Segment`.

use crate::map::Polygon;
use crate::map::Polyline;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Segment;
use std::f64::consts::PI;

/// The millimeter-only derivations [`Segment`] does not carry:
/// sine-arc polylines and corridor polygons - both return map
/// types that stay in this crate.
pub trait SegmentExt {
    /// The sine arc from `from` to `to`, bowed `amplitude` to the
    /// left of the direction at its peak (negative bows right) -
    /// `steps + 1` points including the endpoints.
    fn arc(&self, steps: usize, amplitude: Millimeters) -> Polyline;

    /// A rectangle corridor `half_width` mm to each side of the
    /// segment - the obstacle shape tunnels and decorative
    /// streets block houses with.
    fn corridor(&self, half_width: Millimeters) -> Polygon;
}

impl SegmentExt for Segment<Millimeters> {
    fn arc(&self, steps: usize, amplitude: Millimeters) -> Polyline {
        let direction = self.to - self.from;
        let normal = direction.normalized().perpendicular();
        (0..=steps)
            .map(|step| {
                let t = step as f64 / steps as f64;
                let bend = (PI * t).sin() * amplitude.as_f64();
                self.from + direction * t + normal * bend
            })
            .collect()
    }

    fn corridor(&self, half_width: Millimeters) -> Polygon {
        let direction = self.to - self.from;
        let normal = direction.normalized().perpendicular() * half_width.as_f64();
        Polygon::new(vec![self.from + normal, self.to + normal, self.to - normal, self.from - normal])
    }
}
