//! Decorative map backdrop: water polylines, landmarks and street
//! hints - gameplay-inert geometry for rendering richness.

pub mod kind;
pub mod landmark;
pub mod name;
pub use kind::LandmarkKind;
pub use landmark::Landmark;
pub use name::LandmarkName;
pub use smearor_commons_geometry::Polyline;

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;

/// The decorative backdrop layer of a map dataset.
///
/// Everything here is gameplay-inert: rivers the ferry routes run
/// along, landmark markers for visual orientation, coarse street
/// hints. The `MapStyle` renderer draws it under the station graph;
/// validation only happens inside the domain types themselves.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Backdrop {
    /// Water polylines (Seine, Spree) in map millimeters.
    #[serde(default)]
    pub water: Vec<Polyline<Millimeters>>,
    /// Landmark markers (Eiffel tower, Brandenburger Tor, ...).
    #[serde(default)]
    pub landmarks: Vec<Landmark>,
    /// Coarse street polylines for visual context.
    #[serde(default)]
    pub streets: Vec<Polyline<Millimeters>>,
}

impl Backdrop {
    /// Whether the dataset carries any backdrop geometry.
    pub fn is_empty(&self) -> bool {
        self.water.is_empty() && self.landmarks.is_empty() && self.streets.is_empty()
    }
}
