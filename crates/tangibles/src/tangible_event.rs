use tangible_objects_model::TangibleId;
use tangible_objects_model::TrackedTangible;

/// A tangible lifecycle/motion event from the daemon.
///
/// Mirror of the `tangibleEvents` subscription payload, projected into
/// the app's event vocabulary (spawn/move/remove only - pending
/// hysteresis stays upstream).
#[derive(Debug, Clone)]
pub enum TangibleEvent {
    /// A tangible was placed and confirmed.
    Spawned(TrackedTangible),
    /// A tracked tangible moved or rotated.
    Moved(TrackedTangible),
    /// A tangible was lifted off the surface.
    Removed(TangibleId),
}
