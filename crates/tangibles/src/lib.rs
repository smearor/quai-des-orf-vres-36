//! Tangible input for Quai des Orfèvres 36.
//!
//! Talks to the `tangible-objects` daemon over its GraphQL interface
//! (pattern registration, `tangibleEvents` subscription, `deviceInfo`
//! and `touchPoints` queries) and derives the Eiffel tower anchor pose
//! plus the map-space transform from it.

pub mod anchor_config;
pub mod anchor_pattern;
pub mod anchor_pose;
pub mod anchor_state;
pub mod daemon_error;
pub mod map_scale;
pub mod map_transform;
pub mod surface_mm;
pub mod tangible_event;
pub mod tangible_source;

pub use anchor_config::AnchorConfig;
pub use anchor_pattern::AnchorPattern;
pub use anchor_pose::AnchorPose;
pub use anchor_state::AnchorState;
pub use daemon_error::DaemonError;
pub use map_scale::MapScale;
pub use map_scale::MapScaleError;
pub use map_transform::MapTransform;
pub use surface_mm::SurfaceMm;
pub use tangible_event::TangibleEvent;
pub use tangible_source::TangibleSource;
