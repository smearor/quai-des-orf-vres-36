//! `LayersConfig` - the persisted layer-orientation snapshot in
//! `config.toml`.
//!
//! Every flag is optional so a hand-edited or older config merges
//! into the current defaults; `capture` freezes the live
//! `SceneLayers` for the next save.

use crate::ui::map_scene::LabelMode;
use crate::ui::map_scene::SceneLayers;
use crate::ui::style::HouseMode;
use qdo36_map::LineName;
use qdo36_map::Transport;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;

/// The persistable layer + orientation state (`config.toml`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayersConfig {
    /// Water polylines.
    #[serde(default)]
    pub water: Option<bool>,
    /// City-block layer.
    #[serde(default)]
    pub blocks: Option<bool>,
    /// Backdrop landmark pins.
    #[serde(default)]
    pub landmarks: Option<bool>,
    /// Taxi edges.
    #[serde(default)]
    pub taxi: Option<bool>,
    /// Bus edges.
    #[serde(default)]
    pub bus: Option<bool>,
    /// Metro edges.
    #[serde(default)]
    pub metro: Option<bool>,
    /// Batobus edges.
    #[serde(default)]
    pub batobus: Option<bool>,
    /// Underground edges.
    #[serde(default)]
    pub underground: Option<bool>,
    /// Parallel-transports overlay.
    #[serde(default)]
    pub parallel_transports: Option<bool>,
    /// Highlighted transports (own colors).
    #[serde(default)]
    pub highlight: Vec<Transport>,
    /// Metro lines hidden by name.
    #[serde(default)]
    pub hide_lines: Vec<LineName>,
    /// Station discs.
    #[serde(default)]
    pub stations: Option<bool>,
    /// Station name labels.
    #[serde(default)]
    pub labels: Option<bool>,
    /// Station number labels.
    #[serde(default)]
    pub numbers: Option<bool>,
    /// Importance-score labels.
    #[serde(default)]
    pub scores: Option<bool>,
    /// Station-label orientation.
    #[serde(default)]
    pub label_mode: Option<LabelMode>,
    /// Landmark-pin orientation.
    #[serde(default)]
    pub landmark_mode: Option<LabelMode>,
    /// District-caption orientation.
    #[serde(default)]
    pub district_mode: Option<LabelMode>,
    /// Park-caption orientation.
    #[serde(default)]
    pub park_mode: Option<LabelMode>,
    /// House rendering mode (`radial-iso`/`iso`/`top-down`).
    #[serde(default)]
    pub house_mode: Option<HouseMode>,
    /// The compass rose.
    #[serde(default)]
    pub compass: Option<bool>,
    /// Bridge-station markers.
    #[serde(default)]
    pub bridges: Option<bool>,
}

impl LayersConfig {
    /// Freezes `layers` into the persisted shape.
    pub fn capture(layers: &SceneLayers) -> Self {
        Self {
            water: Some(layers.water),
            blocks: Some(layers.blocks),
            landmarks: Some(layers.landmarks),
            taxi: Some(layers.transports.taxi),
            bus: Some(layers.transports.bus),
            metro: Some(layers.transports.metro),
            batobus: Some(layers.transports.batobus),
            underground: Some(layers.transports.underground),
            parallel_transports: Some(layers.parallel_transports),
            highlight: layers.highlight.iter().copied().collect(),
            hide_lines: layers.hidden_lines.iter().cloned().collect(),
            stations: Some(layers.stations),
            labels: Some(layers.labels),
            numbers: Some(layers.numbers),
            scores: Some(layers.scores),
            label_mode: Some(layers.label_mode),
            landmark_mode: Some(layers.landmark_mode),
            district_mode: Some(layers.district_mode),
            park_mode: Some(layers.park_mode),
            house_mode: Some(layers.house_mode),
            compass: Some(layers.compass),
            bridges: Some(layers.bridges),
        }
    }

    /// Merges the stored flags onto `layers` (`None` keeps the
    /// current value).
    pub fn apply_to(&self, layers: &mut SceneLayers) {
        if let Some(v) = self.water {
            layers.water = v;
        }
        if let Some(v) = self.blocks {
            layers.blocks = v;
        }
        if let Some(v) = self.landmarks {
            layers.landmarks = v;
        }
        if let Some(v) = self.taxi {
            layers.transports.taxi = v;
        }
        if let Some(v) = self.bus {
            layers.transports.bus = v;
        }
        if let Some(v) = self.metro {
            layers.transports.metro = v;
        }
        if let Some(v) = self.batobus {
            layers.transports.batobus = v;
        }
        if let Some(v) = self.underground {
            layers.transports.underground = v;
        }
        if let Some(v) = self.parallel_transports {
            layers.parallel_transports = v;
        }
        layers.highlight = self.highlight.iter().copied().collect::<BTreeSet<_>>();
        layers.hidden_lines = self.hide_lines.iter().cloned().collect();
        if let Some(v) = self.stations {
            layers.stations = v;
        }
        if let Some(v) = self.labels {
            layers.labels = v;
        }
        if let Some(v) = self.numbers {
            layers.numbers = v;
        }
        if let Some(v) = self.scores {
            layers.scores = v;
        }
        if let Some(v) = self.label_mode {
            layers.label_mode = v;
        }
        if let Some(v) = self.landmark_mode {
            layers.landmark_mode = v;
        }
        if let Some(v) = self.district_mode {
            layers.district_mode = v;
        }
        if let Some(v) = self.park_mode {
            layers.park_mode = v;
        }
        if let Some(v) = self.house_mode {
            layers.house_mode = v;
        }
        if let Some(v) = self.compass {
            layers.compass = v;
        }
        if let Some(v) = self.bridges {
            layers.bridges = v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_capture_round_trips_through_toml() {
        let mut layers = SceneLayers {
            water: false,
            district_mode: LabelMode::Screen,
            park_mode: LabelMode::Screen,
            ..SceneLayers::default()
        };
        layers.highlight.insert(Transport::Taxi);
        layers.hidden_lines.insert(LineName::from("M4"));
        let text = toml::to_string(&LayersConfig::capture(&layers)).unwrap();
        let mut restored = SceneLayers::default();
        let config: LayersConfig = toml::from_str(&text).unwrap();
        config.apply_to(&mut restored);
        assert!(!restored.water);
        assert_eq!(restored.district_mode, LabelMode::Screen);
        assert_eq!(restored.park_mode, LabelMode::Screen);
        assert!(restored.highlight.contains(&Transport::Taxi));
        assert!(restored.hidden_lines.contains(&LineName::from("M4")));
    }

    #[test]
    fn missing_flags_keep_the_defaults() {
        let config: LayersConfig = toml::from_str("water = false").unwrap();
        let mut layers = SceneLayers::default();
        config.apply_to(&mut layers);
        assert!(!layers.water);
        assert!(layers.stations);
    }
}
