//! The map-space domain: the affine transform between map and
//! surface millimeters plus its zoom factor.

pub mod scale;
pub mod scale_factor;
pub mod transform;

pub use scale::MapScale;
pub use scale::MapScaleError;
pub use scale_factor::ScaleFactor;
pub use transform::MapTransform;
