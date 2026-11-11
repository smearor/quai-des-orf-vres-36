//! `AnchorTracker` - the anchor state machine with loss hysteresis.

use qdo36_tangibles::AnchorPose;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::TangibleEvent;
use std::time::Duration;
use std::time::Instant;

/// Grace period before a lost anchor falls back to the virtual pose -
/// the flicker-safe hysteresis for detection dropouts.
pub const ANCHOR_LOSS_GRACE: Duration = Duration::from_secs(1);

/// The anchor state machine: tracked pose, virtual fallback and the
/// loss hysteresis for detection dropouts.
///
/// `Added`/`Moved` track the pose; `Removed`/`Lagged` open a loss
/// window but keep the tracked pose until [`Self::settle`] runs past
/// [`ANCHOR_LOSS_GRACE`] - the map keeps rendering while the tower is
/// away (or briefly flickers out).
pub struct AnchorTracker {
    /// The tracked anchor pose, or the virtual fallback.
    anchor: AnchorState,
    /// When the tracked anchor's signal was lost; `None` while a pose
    /// is tracked or no loss window is open.
    lost_since: Option<Instant>,
}

impl AnchorTracker {
    /// A tracker holding `anchor` (usually the virtual fallback).
    pub fn new(anchor: AnchorState) -> Self {
        Self { anchor, lost_since: None }
    }

    /// The current anchor state.
    pub fn state(&self) -> AnchorState {
        self.anchor
    }

    /// Updates the anchor state from a tangible event.
    ///
    /// Returns `true` when a new loss window opened and the caller
    /// should schedule a [`Self::settle`] after [`ANCHOR_LOSS_GRACE`].
    pub fn apply_event(&mut self, event: &TangibleEvent, now: Instant) -> bool {
        match event {
            TangibleEvent::Added(tangible) | TangibleEvent::Moved(tangible) => {
                self.anchor = AnchorState::Tracked(AnchorPose::from(tangible));
                self.lost_since = None;
            }
            TangibleEvent::Removed(_) | TangibleEvent::Lagged(_) => {
                if matches!(self.anchor, AnchorState::Tracked(_)) && self.lost_since.is_none() {
                    self.lost_since = Some(now);
                    return true;
                }
            }
        }
        false
    }

    /// Downgrades a still-lost tracked anchor to the virtual pose once
    /// [`ANCHOR_LOSS_GRACE`] elapsed since the loss. Returns `true`
    /// when the anchor state changed (the scene needs a redraw).
    pub fn settle(&mut self, now: Instant) -> bool {
        match self.lost_since {
            Some(since) if now.saturating_duration_since(since) >= ANCHOR_LOSS_GRACE => self.lost_since = None,
            _ => return false,
        }
        if let AnchorState::Tracked(pose) = self.anchor {
            self.anchor = AnchorState::Virtual(pose);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smearor_commons_geometry::RotationRad;
    use smearor_commons_geometry::Vec2;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PatternName;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TrackedTangible;

    /// A tracked tangible at (`x`, `y`) millimeters - only the pose
    /// matters to the tracker.
    fn tangible(x: f64, y: f64) -> TrackedTangible {
        TrackedTangible {
            id: TangibleId::new(1),
            pattern_id: PatternId::new(1),
            pattern_name: PatternName::new("eifelturm-gross"),
            center: Vec2::new(x, y),
            rotation_rad: RotationRad::new(0.0),
            pending: false,
            pending_frames: FrameCount::new(0),
            spawn_stability: FrameCount::new(0),
            frames_since_match: FrameCount::new(0),
        }
    }

    #[test]
    fn added_tracks_the_pose() {
        let mut tracker = AnchorTracker::new(AnchorState::default_virtual());
        assert!(matches!(tracker.state(), AnchorState::Virtual(_)));

        assert!(!tracker.apply_event(&TangibleEvent::Added(tangible(100.0, 50.0)), Instant::now()));
        assert!(matches!(tracker.state(), AnchorState::Tracked(pose) if pose.center.x == 100.0 && pose.center.y == 50.0));
    }

    #[test]
    fn loss_opens_a_window_but_stays_tracked() {
        let t0 = Instant::now();
        let mut tracker = AnchorTracker::new(AnchorState::default_virtual());
        tracker.apply_event(&TangibleEvent::Added(tangible(100.0, 50.0)), t0);

        assert!(tracker.apply_event(&TangibleEvent::Removed(TangibleId::new(1)), t0));
        assert!(matches!(tracker.state(), AnchorState::Tracked(_)));

        // Settling inside the grace period changes nothing.
        assert!(!tracker.settle(t0 + ANCHOR_LOSS_GRACE / 2));
        assert!(matches!(tracker.state(), AnchorState::Tracked(_)));
    }

    #[test]
    fn settle_after_grace_falls_back_to_virtual() {
        let t0 = Instant::now();
        let mut tracker = AnchorTracker::new(AnchorState::default_virtual());
        tracker.apply_event(&TangibleEvent::Added(tangible(100.0, 50.0)), t0);
        tracker.apply_event(&TangibleEvent::Removed(TangibleId::new(1)), t0);

        assert!(tracker.settle(t0 + ANCHOR_LOSS_GRACE));
        // The virtual fallback keeps the last pose (the map stays put).
        assert!(matches!(tracker.state(), AnchorState::Virtual(pose) if pose.center.x == 100.0));

        // A second settle is a no-op.
        assert!(!tracker.settle(t0 + 2 * ANCHOR_LOSS_GRACE));
    }

    #[test]
    fn retracking_closes_the_window() {
        let t0 = Instant::now();
        let mut tracker = AnchorTracker::new(AnchorState::default_virtual());
        tracker.apply_event(&TangibleEvent::Added(tangible(100.0, 50.0)), t0);
        tracker.apply_event(&TangibleEvent::Removed(TangibleId::new(1)), t0);

        // The tower comes back inside the grace period.
        assert!(!tracker.apply_event(&TangibleEvent::Moved(tangible(110.0, 50.0)), t0 + ANCHOR_LOSS_GRACE / 2));
        assert!(!tracker.settle(t0 + ANCHOR_LOSS_GRACE));
        assert!(matches!(tracker.state(), AnchorState::Tracked(pose) if pose.center.x == 110.0));
    }

    #[test]
    fn lagged_behaves_like_removed() {
        let t0 = Instant::now();
        let mut tracker = AnchorTracker::new(AnchorState::default_virtual());
        tracker.apply_event(&TangibleEvent::Moved(tangible(100.0, 50.0)), t0);

        assert!(tracker.apply_event(&TangibleEvent::Lagged(None), t0));
        assert!(tracker.settle(t0 + ANCHOR_LOSS_GRACE));
        assert!(matches!(tracker.state(), AnchorState::Virtual(_)));
    }

    #[test]
    fn loss_while_virtual_is_a_noop() {
        let t0 = Instant::now();
        let mut tracker = AnchorTracker::new(AnchorState::default_virtual());

        assert!(!tracker.apply_event(&TangibleEvent::Removed(TangibleId::new(1)), t0));
        assert!(!tracker.settle(t0 + ANCHOR_LOSS_GRACE));
        assert!(matches!(tracker.state(), AnchorState::Virtual(_)));
    }
}
