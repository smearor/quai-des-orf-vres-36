//! `PxPolygon` - a px-space polygon view over a position slice.

use crate::ui::style::renderer::PxPos;
use std::ops::Deref;
use tangible_objects_model::Vec2;

/// A px-space closed ring over a borrowed `PxPos` slice - the
/// render counterpart of `Polygon`'s ring queries (`segments`,
/// `centroid`), without owning the points.
#[derive(Debug, Clone, Copy)]
pub struct PxPolygon<'a>(pub &'a [PxPos]);

impl PxPolygon<'_> {
    /// The ring's vertex centroid in widget px.
    pub fn centroid(&self) -> PxPos {
        let count = self.0.len().max(1) as f64;
        Vec2::new(self.0.iter().map(|p| p.x).sum::<f64>() / count, self.0.iter().map(|p| p.y).sum::<f64>() / count)
    }

    /// The ring's consecutive edge segments (`(i, i + 1)`,
    /// wrapping the end back to the start).
    pub fn segments(&self) -> impl Iterator<Item = (PxPos, PxPos)> + '_ {
        let n = self.0.len();
        (0..n).map(move |i| (self.0[i], self.0[(i + 1) % n]))
    }

    /// The polygon shrunk toward its centroid by `fraction` -
    /// a px-space `Polygon::offset` light (roof bodies, insets).
    pub fn shrunk(&self, fraction: f64) -> Vec<PxPos> {
        let c = self.centroid();
        self.0
            .iter()
            .map(|p| Vec2::new(p.x + (c.x - p.x) * fraction, p.y + (c.y - p.y) * fraction))
            .collect()
    }
}

impl Deref for PxPolygon<'_> {
    type Target = [PxPos];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
