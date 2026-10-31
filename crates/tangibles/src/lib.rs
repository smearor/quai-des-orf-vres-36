//! Tangible input for Quai des Orfèvres 36.
//!
//! Talks to the `tangible-objects` daemon over its GraphQL interface
//! (pattern registration, `tangibleEvents` subscription, `deviceInfo`
//! and `touchPoints` queries) and derives the landmark anchor pose
//! plus the map-space transform from it.

pub mod anchor;
pub mod map;
pub mod source;

pub use anchor::AnchorConfig;
pub use anchor::AnchorPattern;
pub use anchor::AnchorPose;
pub use anchor::AnchorState;
pub use map::MapScale;
pub use map::MapScaleError;
pub use map::MapTransform;
pub use source::DaemonError;
pub use source::SurfaceMm;
pub use source::TangibleEvent;
pub use source::TangibleSource;
