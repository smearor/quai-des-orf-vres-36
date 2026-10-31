use tangible_objects_model::EventCount;
use tangible_objects_model::EventKind;
use tangible_objects_model::TangibleId;
use tangible_objects_model::TrackedTangible;

/// A tangible lifecycle/motion event from the daemon.
///
/// Projection of the upstream `tangibleEvents` subscription payload:
/// tangible spawn/move/remove plus the `LAGGED` control event (a
/// resync hint, never a tangible change).
#[derive(Debug, Clone)]
pub enum TangibleEvent {
    /// A tangible was placed and confirmed.
    Added(TrackedTangible),
    /// A tracked tangible moved or rotated.
    Moved(TrackedTangible),
    /// A tangible was lifted off the surface.
    Removed(TangibleId),
    /// The subscription skipped events - resync via `tangibles()`.
    Lagged(Option<EventCount>),
}

impl TangibleEvent {
    /// A synthetic `Lagged` event (stream error or upstream skip).
    pub fn lagged(skipped: Option<EventCount>) -> Self {
        Self::Lagged(skipped)
    }

    /// Projects an upstream event into the app's vocabulary.
    ///
    /// Returns `None` for kinds that do not concern tracked tangibles
    /// (field/set/lifecycle events).
    pub fn project(event: &tangible_objects_model::TangibleEvent) -> Option<Self> {
        match event.kind {
            EventKind::TangibleAdded | EventKind::ObjectAdded => event.tangible.clone().map(Self::Added),
            EventKind::TangibleMoved | EventKind::ObjectMoved => event.tangible.clone().map(Self::Moved),
            EventKind::TangibleRemoved | EventKind::ObjectRemoved => event.tangible.as_ref().map(|t| Self::Removed(t.id)),
            EventKind::Lagged => Some(Self::Lagged(event.skipped)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tangible_objects_model::TangibleEvent as UpstreamEvent;

    fn upstream(kind: EventKind) -> UpstreamEvent {
        UpstreamEvent {
            kind,
            tangible: None,
            set_id: None,
            field_set: None,
            field: None,
            count: None,
            frames: None,
            pattern_id: None,
            name: None,
            owner: None,
            detail: None,
            skipped: None,
        }
    }

    #[test]
    fn lagged_projects_skipped_count() {
        let mut event = upstream(EventKind::Lagged);
        event.skipped = Some(EventCount::new(3));
        let Some(TangibleEvent::Lagged(Some(count))) = TangibleEvent::project(&event) else {
            panic!("expected Lagged(Some(_))");
        };
        assert_eq!(count.as_u64(), 3);
    }

    #[test]
    fn non_tangible_kinds_project_to_none() {
        assert!(TangibleEvent::project(&upstream(EventKind::FieldPlaced)).is_none());
        assert!(TangibleEvent::project(&upstream(EventKind::ProfileRegistered)).is_none());
        assert!(TangibleEvent::project(&upstream(EventKind::TouchPointsUpdated)).is_none());
    }

    #[test]
    fn tangible_kinds_without_payload_project_to_none() {
        // A TANGIBLE_* event without a `tangible` payload is dropped,
        // not mapped to a bogus event.
        assert!(TangibleEvent::project(&upstream(EventKind::TangibleAdded)).is_none());
        assert!(TangibleEvent::project(&upstream(EventKind::TangibleRemoved)).is_none());
    }
}
