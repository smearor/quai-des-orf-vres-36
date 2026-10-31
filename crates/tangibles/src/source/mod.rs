//! The daemon seam: the [`TangibleSource`] trait, its events and the
//! surface/touch vocabulary coming over the wire.

pub mod error;
pub mod event;
pub mod graphql_source;
pub mod surface;

pub use error::DaemonError;
pub use event::TangibleEvent;
pub use graphql_source::GraphQlSource;
pub use surface::SurfaceMm;

use crate::AnchorPattern;
use async_channel::Receiver;
use std::future::Future;
use tangible_objects_model::PatternId;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

/// Registration owner stamped on runtime patterns/field sets, used
/// for `unregisterOwner` cleanup on shutdown.
pub const REGISTRATION_OWNER: &str = "qdo36";

/// Abstraction over the tangible-objects daemon.
///
/// Same seam as tangibles-chess's `TangibleSource`: the app talks to
/// this trait, `GraphQlSource` implements it against a real daemon and
/// a mock serves tests/dev without hardware.
pub trait TangibleSource: Send {
    /// Registers the anchor profile; returns its pattern id.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Registration`] on rejection or connection loss.
    fn register_anchor_profile(&mut self, profile: &AnchorPattern) -> impl Future<Output = Result<PatternId, DaemonError>> + Send;

    /// Stream of tangible lifecycle/motion events for one pattern.
    fn tangible_events(&self, pattern: PatternId) -> impl Future<Output = Receiver<TangibleEvent>> + Send;

    /// Snapshot of all currently tracked tangibles.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Query`] on a failed query.
    fn tangibles(&self) -> impl Future<Output = Result<Vec<TrackedTangible>, DaemonError>> + Send;

    /// Snapshot of raw active contacts.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Query`] on a failed query.
    fn touch_points(&self) -> impl Future<Output = Result<Vec<TouchPoint>, DaemonError>> + Send;

    /// Surface geometry from `deviceInfo`.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Query`] on a failed query.
    fn surface(&self) -> impl Future<Output = Result<SurfaceMm, DaemonError>> + Send;

    /// Removes all registrations owned by this application
    /// (`unregisterOwner`) - session cleanup on shutdown.
    ///
    /// # Errors
    ///
    /// [`DaemonError::Query`] on a failed mutation.
    fn unregister(&mut self) -> impl Future<Output = Result<(), DaemonError>> + Send;
}
