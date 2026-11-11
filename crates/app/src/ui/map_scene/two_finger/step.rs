//! `TwoFingerStep` - one applied two-finger step per moved touch point.

use crate::ui::style::PxPos;
use qdo36_tangibles::ScaleFactor;
use smearor_commons_geometry::RotationRad;

/// One applied two-finger step, emitted per moved touch point once
/// the session mode locked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TwoFingerStep {
    /// Move: pan the old centroid onto the new one, pinch-zoom by the
    /// span factor.
    Move {
        /// Pair midpoint before the move (widget px).
        centroid_before: PxPos,
        /// Pair midpoint after the move (widget px).
        centroid_after: PxPos,
        /// `span_after / span_before` pinch factor (ONE = no zoom).
        span_factor: ScaleFactor,
    },
    /// Rotate: turn the anchor pose around the current pair centroid.
    Rotate {
        /// Rotation pivot (widget px) - the current pair midpoint; a
        /// zero delta re-pins the same point and pans nothing.
        pivot: PxPos,
        /// Incremental line-angle change.
        delta_rad: RotationRad,
    },
}
