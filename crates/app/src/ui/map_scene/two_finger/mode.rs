//! `TwoFingerMode` - the locked intent of a two-finger session.

/// The locked intent of a two-finger session: the first metric to
/// cross its threshold decides, the lock holds until the pair breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoFingerMode {
    /// Not yet decided - sub-threshold motion emits no step.
    Undecided,
    /// Move: the centroid shifts the view pan, the span pinch-zooms.
    Move,
    /// Rotate: the finger-line angle turns the virtual anchor pose.
    Rotate,
}
