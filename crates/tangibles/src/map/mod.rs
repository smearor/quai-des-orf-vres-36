//! The map-space domain: the affine transform between map and
//! surface millimeters plus its zoom factor.

pub mod scale;
pub mod transform;

pub use scale::MapScale;
pub use scale::MapScaleError;
pub use transform::MapTransform;
