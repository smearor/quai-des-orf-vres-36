//! The anchor domain: the landmark tangible that fixes and steers the
//! map (profile, measured pose, calibration config, smoothed state).

pub mod config;
pub mod pattern;
pub mod pose;
pub mod state;

pub use config::AnchorConfig;
pub use pattern::AnchorPattern;
pub use pose::AnchorPose;
pub use state::AnchorState;
