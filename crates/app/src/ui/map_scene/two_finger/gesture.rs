//! `TwoFingerGesture` - mode lock and step extraction for the
//! two-finger pan/pinch/rotate session.

use crate::ui::map_scene::two_finger::TwoFingerMode;
use crate::ui::map_scene::two_finger::TwoFingerStep;
use crate::ui::style::PxPos;
use qdo36_tangibles::ScaleFactor;
use std::f64::consts::FRAC_PI_2;
use std::f64::consts::PI;
use tangible_objects_model::Pixels;
use tangible_objects_model::RotationRad;
use tangible_objects_model::Vec2;

/// Centroid travel locking the session into move mode.
const MOVE_LOCK: Pixels = Pixels::new(8.0);
/// Span change locking the session into move mode.
const SPAN_LOCK: Pixels = Pixels::new(12.0);
/// Line rotation locking the session into rotate mode.
const ROTATE_LOCK: RotationRad = RotationRad::new(0.15);

/// The two-finger session state: the undecided/move/rotate lock and
/// the pair as it looked when the second finger landed - the
/// reference frame for the lock thresholds and the rotation pivot.
pub struct TwoFingerGesture {
    /// The locked gesture mode for the current pair.
    mode: TwoFingerMode,
    /// The pair positions at second-finger-down; `None` while fewer
    /// (or more) than two touches rest on the surface.
    reference: Option<(PxPos, PxPos)>,
}

impl Default for TwoFingerGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl TwoFingerGesture {
    /// A session starting undecided.
    pub fn new() -> Self {
        Self {
            mode: TwoFingerMode::Undecided,
            reference: None,
        }
    }

    /// The pair changed - the second finger landed or a lift left a
    /// remaining pair; the lock resets and the reference re-bases.
    pub fn pair_started(&mut self, first: PxPos, second: PxPos) {
        self.mode = TwoFingerMode::Undecided;
        self.reference = Some((first, second));
    }

    /// The pair broke (a third finger or a lift leaving one) - lock
    /// and reference reset.
    pub fn pair_broken(&mut self) {
        self.mode = TwoFingerMode::Undecided;
        self.reference = None;
    }

    /// One finger moved `before` -> `after`; `other` is the resting
    /// finger. Emits the next [`TwoFingerStep`] once the mode locked.
    pub fn update(&mut self, before: PxPos, after: PxPos, other: PxPos) -> Option<TwoFingerStep> {
        self.reference?;
        if self.mode == TwoFingerMode::Undecided {
            self.mode = self.decide(after, other);
            if self.mode == TwoFingerMode::Undecided {
                return None;
            }
        }
        match self.mode {
            TwoFingerMode::Move => Some(TwoFingerStep::Move {
                centroid_before: Vec2::centroid(&[before, other]),
                centroid_after: Vec2::centroid(&[after, other]),
                span_factor: ScaleFactor::between(before.distance(&other), after.distance(&other)),
            }),
            TwoFingerMode::Rotate => Some(TwoFingerStep::Rotate {
                pivot: Vec2::centroid(&[after, other]),
                delta_rad: wrap_half_pi(line_angle(other, after) - line_angle(other, before)),
            }),
            TwoFingerMode::Undecided => None,
        }
    }

    /// Locks the mode once an accumulated metric crosses its
    /// threshold: line rotation -> [`TwoFingerMode::Rotate`],
    /// centroid travel or span change -> [`TwoFingerMode::Move`];
    /// rotation wins a tie in the same update.
    fn decide(&self, after: PxPos, other: PxPos) -> TwoFingerMode {
        let Some((first, second)) = self.reference else {
            return TwoFingerMode::Undecided;
        };
        let turned = wrap_half_pi(line_angle(other, after) - line_angle(first, second));
        if turned.as_f64().abs() > ROTATE_LOCK.as_f64() {
            return TwoFingerMode::Rotate;
        }
        let moved = Vec2::centroid(&[first, second]).distance(&Vec2::centroid(&[after, other]));
        let spanned = (after.distance(&other) - first.distance(&second)).abs();
        if moved > MOVE_LOCK || spanned > SPAN_LOCK {
            return TwoFingerMode::Move;
        }
        TwoFingerMode::Undecided
    }
}

/// The line direction as an undirected axis in [-pi/2, pi/2) -
/// either endpoint may play `after`/`other`.
fn line_angle(from: PxPos, to: PxPos) -> RotationRad {
    (to - from).direction().axis()
}

/// Wraps an angle delta into [-pi/2, pi/2] (line angles live mod pi).
fn wrap_half_pi(delta: RotationRad) -> RotationRad {
    RotationRad::new((delta.as_f64() + FRAC_PI_2).rem_euclid(PI) - FRAC_PI_2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two fingers 100 px apart horizontally, centroid at (150, 100).
    fn pair() -> (TwoFingerGesture, PxPos, PxPos) {
        let mut gesture = TwoFingerGesture::new();
        let (first, second) = (Vec2::new(100.0, 100.0), Vec2::new(200.0, 100.0));
        gesture.pair_started(first, second);
        (gesture, first, second)
    }

    #[test]
    fn sub_threshold_motion_stays_undecided() {
        let (mut gesture, first, second) = pair();
        assert_eq!(gesture.update(first, Vec2::new(103.0, 100.0), second), None);
    }

    #[test]
    fn centroid_travel_locks_move() {
        let (mut gesture, first, second) = pair();
        let step = gesture.update(first, Vec2::new(120.0, 100.0), second);
        assert!(matches!(step, Some(TwoFingerStep::Move { centroid_before, centroid_after, .. })
            if centroid_before.x == 150.0 && centroid_after.x == 160.0));
    }

    #[test]
    fn span_change_locks_move_with_the_pinch_factor() {
        let (mut gesture, first, second) = pair();
        let step = gesture.update(first, Vec2::new(120.0, 100.0), second);
        assert!(matches!(step, Some(TwoFingerStep::Move { span_factor, .. }) if (span_factor.get() - 0.8).abs() < 1e-9));
    }

    #[test]
    fn line_rotation_locks_rotate_around_the_current_centroid() {
        let (mut gesture, first, second) = pair();
        // Rotate the pair ~20 degrees around the second finger.
        let step = gesture.update(first, Vec2::new(106.0, 66.0), second);
        match step {
            Some(TwoFingerStep::Rotate { pivot, delta_rad }) => {
                assert_eq!(pivot, Vec2::new(153.0, 83.0));
                assert!(delta_rad.as_f64() > 0.3 && delta_rad.as_f64() < 0.4, "delta_rad {delta_rad:?}");
            }
            other => panic!("expected rotate step, got {other:?}"),
        }
    }

    #[test]
    fn the_other_endpoint_rotates_the_same() {
        let (mut gesture, first, second) = pair();
        // Same ~20 degree rotation, but the second finger moves.
        let step = gesture.update(second, Vec2::new(194.0, 134.0), first);
        assert!(matches!(step, Some(TwoFingerStep::Rotate { delta_rad, .. }) if delta_rad.as_f64() > 0.3 && delta_rad.as_f64() < 0.4));
    }

    #[test]
    fn a_move_lock_survives_later_rotation() {
        let (mut gesture, first, second) = pair();
        gesture.update(first, Vec2::new(130.0, 100.0), second);
        // A rotation attempt now still yields move steps.
        let step = gesture.update(Vec2::new(130.0, 100.0), Vec2::new(130.0, 70.0), second);
        assert!(matches!(step, Some(TwoFingerStep::Move { .. })));
    }

    #[test]
    fn a_rotate_lock_never_pans() {
        let (mut gesture, first, second) = pair();
        gesture.update(first, Vec2::new(106.0, 66.0), second);
        // A real parallel translation arrives as two single-finger
        // updates: each stays a rotate step (never move) and the two
        // skew deltas cancel out to zero net rotation.
        let first_step = gesture.update(Vec2::new(106.0, 66.0), Vec2::new(156.0, 66.0), second);
        let second_step = gesture.update(second, Vec2::new(250.0, 100.0), Vec2::new(156.0, 66.0));
        let net = match (first_step, second_step) {
            (Some(TwoFingerStep::Rotate { delta_rad: first, .. }), Some(TwoFingerStep::Rotate { delta_rad: second, .. })) => first + second,
            other => panic!("expected two rotate steps, got {other:?}"),
        };
        assert!(net.as_f64().abs() < 1e-9, "net rotation {net:?}");
    }

    #[test]
    fn pair_broken_resets_the_lock() {
        let (mut gesture, first, second) = pair();
        gesture.update(first, Vec2::new(130.0, 100.0), second);
        gesture.pair_broken();
        assert_eq!(gesture.update(Vec2::new(130.0, 100.0), Vec2::new(150.0, 100.0), second), None);
    }

    #[test]
    fn a_restarted_pair_redecides() {
        let (mut gesture, _, _) = pair();
        gesture.pair_started(Vec2::new(100.0, 100.0), Vec2::new(200.0, 100.0));
        let step = gesture.update(Vec2::new(100.0, 100.0), Vec2::new(106.0, 66.0), Vec2::new(200.0, 100.0));
        assert!(matches!(step, Some(TwoFingerStep::Rotate { .. })));
    }

    #[test]
    fn pinching_a_touching_pair_is_a_noop() {
        let mut gesture = TwoFingerGesture::new();
        let (first, second) = (Vec2::new(150.0, 100.0), Vec2::new(150.5, 100.0));
        gesture.pair_started(first, second);
        // The 50 px span change locks move; the sub-px span guard
        // keeps the factor at 1.0 instead of dividing by ~zero.
        let step = gesture.update(first, Vec2::new(100.0, 100.0), second);
        assert!(matches!(step, Some(TwoFingerStep::Move { span_factor, .. }) if span_factor == ScaleFactor::ONE));
    }
}
