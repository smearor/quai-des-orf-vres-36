use crate::AnchorPattern;
use crate::DaemonError;
use crate::SurfaceMm;
use crate::TangibleEvent;
use tangible_objects_model::PatternId;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

/// Abstraction over the tangible-objects daemon.
///
/// Same seam as tangibles-chess's `TangibleSource`: the app talks to
/// this trait, `GraphQlSource` implements it against a real daemon and
/// a mock serves tests/dev without hardware.
pub trait TangibleSource: Send {
    /// Registers the Eiffel tower profile; returns its pattern id.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Registration`] on rejection or connection loss.
    fn register_anchor_profile(&mut self, profile: &AnchorPattern) -> impl std::future::Future<Output = Result<PatternId, DaemonError>> + Send;

    /// Stream of tangible lifecycle/motion events for one pattern.
    fn tangible_events(&self, pattern: PatternId) -> impl std::future::Future<Output = async_channel::Receiver<TangibleEvent>> + Send;

    /// Snapshot of all currently tracked tangibles.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Query`] on a failed query.
    fn tangibles(&self) -> impl std::future::Future<Output = Result<Vec<TrackedTangible>, DaemonError>> + Send;

    /// Snapshot of raw active contacts.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Query`] on a failed query.
    fn touch_points(&self) -> impl std::future::Future<Output = Result<Vec<TouchPoint>, DaemonError>> + Send;

    /// Surface geometry from `deviceInfo`.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Query`] on a failed query.
    fn surface(&self) -> impl std::future::Future<Output = Result<SurfaceMm, DaemonError>> + Send;
}
