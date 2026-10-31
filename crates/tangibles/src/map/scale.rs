use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use thiserror::Error;

/// Map zoom factor: surface millimeters per map millimeter.
///
/// The physical table is a viewport over a (potentially larger)
/// virtual map - `MapScale` is the zoom knob the overlay controls turn.
/// At 1.0 one map mm is one surface mm; larger values magnify, so the
/// map may exceed the visible area and the anchor pans the viewport.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapScale(f64);

impl MapScale {
    /// Smallest usable zoom (map shrunk to half-size on the table).
    pub const MIN: MapScale = MapScale(0.5);
    /// Largest usable zoom (a 600 mm map spans 3 m of table).
    pub const MAX: MapScale = MapScale(5.0);
    /// Neutral zoom.
    pub const ONE: MapScale = MapScale(1.0);
    /// Zoom step factor applied by the overlay's +/- controls.
    const STEP: f64 = 1.25;

    /// A scale factor; must be finite and within `[MIN, MAX]`.
    ///
    /// # Errors
    ///
    /// [`MapScaleError`] when the value is non-finite or out of range.
    pub fn try_new(factor: f64) -> Result<Self, MapScaleError> {
        if factor.is_finite() && (Self::MIN.0..=Self::MAX.0).contains(&factor) {
            Ok(Self(factor))
        } else {
            Err(MapScaleError {
                factor,
                min: Self::MIN.factor(),
                max: Self::MAX.factor(),
            })
        }
    }

    /// The raw factor.
    pub const fn factor(self) -> f64 {
        self.0
    }

    /// One zoom step in (×1.25), clamped at [`Self::MAX`].
    pub fn zoomed_in(self) -> Self {
        Self(self.0 * Self::STEP).min(Self::MAX)
    }

    /// One zoom step out (÷1.25), clamped at [`Self::MIN`].
    pub fn zoomed_out(self) -> Self {
        Self(self.0 / Self::STEP).max(Self::MIN)
    }

    fn min(self, other: Self) -> Self {
        if self.0 < other.0 { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self.0 > other.0 { self } else { other }
    }
}

impl Default for MapScale {
    /// Full-map fit for the current datasets (~2.0 on a 65" surface).
    fn default() -> Self {
        Self(2.0)
    }
}

impl fmt::Display for MapScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}x", self.0)
    }
}

/// A zoom factor that is non-finite or outside `[MapScale::MIN, MapScale::MAX]`.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("map scale {factor} must be finite and in [{min}, {max}]")]
pub struct MapScaleError {
    /// The rejected factor.
    pub factor: f64,
    /// Minimum allowed factor.
    pub min: f64,
    /// Maximum allowed factor.
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_steps_clamp() {
        let scale = MapScale::default();
        assert_eq!(scale.zoomed_in().factor(), 2.5);
        assert_eq!(scale.zoomed_out().factor(), 1.6);
        let mut max = scale;
        for _ in 0..10 {
            max = max.zoomed_in();
        }
        assert_eq!(max, MapScale::MAX);
        let mut min = scale;
        for _ in 0..10 {
            min = min.zoomed_out();
        }
        assert_eq!(min, MapScale::MIN);
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(MapScale::try_new(0.0).is_err());
        assert!(MapScale::try_new(f64::NAN).is_err());
        assert!(MapScale::try_new(9.9).is_err());
        assert!(MapScale::try_new(2.1).is_ok());
    }
}
