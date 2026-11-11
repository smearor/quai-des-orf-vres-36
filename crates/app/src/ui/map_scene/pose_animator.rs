//! `PoseAnimator` - smooth transitions between anchor poses.
//!
//! Sits between the logical `AnchorState` (tracked / virtual) and the
//! rendered pose: small per-event deltas are followed with a
//! frame-rate independent exponential blend (tracking mode), while
//! large jumps - re-placement elsewhere or the first pose after the
//! virtual fallback - run a timed ease (teleport mode) so the map
//! visibly slides and rotates into place.

use qdo36_tangibles::AnchorPose;
use smearor_commons_geometry::Millimeters;
use std::f64::consts::PI;
use std::time::Duration;
use std::time::Instant;

/// Duration of the visible ease for teleport jumps.
const TRANSITION_DURATION: Duration = Duration::from_millis(300);

/// Center jump beyond which a pose change counts as teleport.
const TELEPORT_DISTANCE_MILLIMETERS: f64 = 80.0;

/// Rotation jump beyond which a pose change counts as teleport (~25°).
const TELEPORT_ROTATION_RAD: f64 = 25.0 * PI / 180.0;

/// Tracking follow time constant: the rendered pose closes ~63% of
/// the remaining gap per this interval.
const TRACKING_TAU: Duration = Duration::from_millis(80);

/// Residual center gap below which tracking snaps to the target.
const SETTLE_DISTANCE_MILLIMETERS: f64 = 0.5;

/// Residual rotation gap below which tracking snaps to the target.
const SETTLE_ROTATION_RAD: f64 = 0.005;

/// Interpolates the rendered anchor pose toward the latest target.
///
/// The widget calls [`Self::set_target`] on every incoming pose and
/// [`Self::pose_at`] once per frame; [`Self::is_animating`] tells it
/// whether further frames are due.
#[derive(Debug)]
pub struct PoseAnimator {
    /// The pose rendered this frame.
    rendered: AnchorPose,
    /// The latest pose reported by the tangible stream.
    target: AnchorPose,
    /// Timed teleport ease: start pose and start instant; `target` is
    /// the destination. `None` in tracking mode.
    transition: Option<(AnchorPose, Instant)>,
    /// When `pose_at` last ran - the tracking follow's frame delta.
    last_frame: Instant,
}

impl PoseAnimator {
    /// An animator holding `pose`, without an initial transition.
    pub fn new(pose: AnchorPose, now: Instant) -> Self {
        Self {
            rendered: pose,
            target: pose,
            transition: None,
            last_frame: now,
        }
    }

    /// Records a new target pose.
    ///
    /// A jump beyond the teleport thresholds (measured against the
    /// previous target, so poses arriving mid-ease do not restart it)
    /// opens a timed ease from the rendered pose; smaller deltas are
    /// left to the tracking follow.
    pub fn set_target(&mut self, target: AnchorPose, now: Instant) {
        let teleported = self.target.center.distance(&target.center) >= Millimeters::new(TELEPORT_DISTANCE_MILLIMETERS)
            || self.target.rotation_rad.signed_delta_to(target.rotation_rad).as_f64().abs() >= TELEPORT_ROTATION_RAD;
        if teleported {
            self.transition = Some((self.rendered, now));
        }
        self.target = target;
    }

    /// The pose to render at `now` - advances the teleport ease or the
    /// tracking follow toward the target.
    pub fn pose_at(&mut self, now: Instant) -> AnchorPose {
        match self.transition {
            Some((from, started)) => {
                let progress = (now.saturating_duration_since(started).as_secs_f64() / TRANSITION_DURATION.as_secs_f64()).min(1.0);
                if progress >= 1.0 {
                    self.transition = None;
                    self.rendered = self.target;
                } else {
                    self.rendered = from.lerp(self.target, ease_out_cubic(progress));
                }
            }
            None => {
                let alpha = 1.0 - (-now.saturating_duration_since(self.last_frame).as_secs_f64() / TRACKING_TAU.as_secs_f64()).exp();
                self.rendered = self.rendered.lerp(self.target, alpha);
                if self.settled() {
                    self.rendered = self.target;
                }
            }
        }
        self.last_frame = now;
        self.rendered
    }

    /// Whether the rendered pose still differs from the target - the
    /// widget redraws while animating.
    pub fn is_animating(&self) -> bool {
        self.transition.is_some() || !self.settled()
    }

    /// Whether the rendered pose has reached the target within the
    /// settle tolerances.
    fn settled(&self) -> bool {
        self.rendered.center.distance(&self.target.center) <= Millimeters::new(SETTLE_DISTANCE_MILLIMETERS)
            && self.rendered.rotation_rad.signed_delta_to(self.target.rotation_rad).as_f64().abs() <= SETTLE_ROTATION_RAD
    }
}

/// Cubic ease-out: fast start, gentle settle.
fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use smearor_commons_geometry::RotationRad;
    use smearor_commons_geometry::Vec2;

    fn pose(x: f64, y: f64, rotation_rad: f64) -> AnchorPose {
        AnchorPose {
            center: Vec2::new(x, y),
            rotation_rad: RotationRad::new(rotation_rad),
        }
    }

    #[test]
    fn small_moves_track_toward_the_target() {
        let t0 = Instant::now();
        let mut animator = PoseAnimator::new(pose(0.0, 0.0, 0.0), t0);
        animator.set_target(pose(10.0, 0.0, 0.0), t0);

        // One 16 ms frame: ~18% of the gap closes (EMA, tau = 80 ms).
        let rendered = animator.pose_at(t0 + Duration::from_millis(16));
        assert!(rendered.center.x > 0.5 && rendered.center.x < 5.0);
        assert!(animator.is_animating());

        // Long enough later the pose settles on the target and idles.
        let rendered = animator.pose_at(t0 + Duration::from_secs(1));
        assert_eq!(rendered.center.x, 10.0);
        assert!(!animator.is_animating());
    }

    #[test]
    fn large_jumps_run_a_timed_ease() {
        let t0 = Instant::now();
        let mut animator = PoseAnimator::new(pose(0.0, 0.0, 0.0), t0);
        animator.set_target(pose(500.0, 0.0, 0.0), t0);

        // Start of the ease: still the rendered pose.
        assert_eq!(animator.pose_at(t0).center.x, 0.0);

        // Half-way through the 300 ms ease: past the midpoint
        // (ease-out cubic reaches 87.5%).
        let rendered = animator.pose_at(t0 + Duration::from_millis(150));
        assert!(rendered.center.x > 250.0 && rendered.center.x < 500.0);
        assert!(animator.is_animating());

        // Done: exactly the target, no further redraws.
        let rendered = animator.pose_at(t0 + TRANSITION_DURATION);
        assert_eq!(rendered.center.x, 500.0);
        assert!(!animator.is_animating());
    }

    #[test]
    fn mid_ease_targets_do_not_restart_the_transition() {
        let t0 = Instant::now();
        let mut animator = PoseAnimator::new(pose(0.0, 0.0, 0.0), t0);
        animator.set_target(pose(500.0, 0.0, 0.0), t0);
        animator.pose_at(t0 + Duration::from_millis(150));

        // Small deltas near the new target keep the ease running and
        // bend it toward the latest pose instead of restarting it.
        animator.set_target(pose(510.0, 0.0, 0.0), t0 + Duration::from_millis(200));
        let rendered = animator.pose_at(t0 + TRANSITION_DURATION);
        assert_eq!(rendered.center.x, 510.0);
        assert!(!animator.is_animating());
    }

    #[test]
    fn rotation_takes_the_shortest_arc() {
        let t0 = Instant::now();
        let mut animator = PoseAnimator::new(pose(0.0, 0.0, 2.9), t0);
        // Wrapped delta: +0.48 rad across ±PI, not -5.8 the long way.
        animator.set_target(pose(0.0, 0.0, -2.9), t0);

        let rendered = animator.pose_at(t0 + Duration::from_millis(150));
        assert!(rendered.rotation_rad.as_f64() > 2.9);
    }

    #[test]
    fn retarget_during_ease_continues_smoothly() {
        let t0 = Instant::now();
        let mut animator = PoseAnimator::new(pose(0.0, 0.0, 0.0), t0);
        animator.set_target(pose(500.0, 0.0, 0.0), t0);
        let midway = animator.pose_at(t0 + Duration::from_millis(150));

        // A second teleport re-eases from the rendered pose, not the
        // stale first target.
        animator.set_target(pose(700.0, 0.0, 0.0), t0 + Duration::from_millis(150));
        let rendered = animator.pose_at(t0 + Duration::from_millis(150));
        assert_eq!(rendered.center.x, midway.center.x);
        let rendered = animator.pose_at(t0 + Duration::from_millis(450));
        assert_eq!(rendered.center.x, 700.0);
    }
}
