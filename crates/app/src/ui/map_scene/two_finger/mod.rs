//! Two-finger pan/pinch/rotate handling for the virtual-anchor view:
//! the session locks into move or rotate on the first threshold
//! crossing and emits one [`TwoFingerStep`] per moved touch point.

pub mod gesture;
pub mod mode;
pub mod step;

pub use gesture::TwoFingerGesture;
pub use mode::TwoFingerMode;
pub use step::TwoFingerStep;
