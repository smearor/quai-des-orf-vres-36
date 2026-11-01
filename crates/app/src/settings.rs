use std::path::PathBuf;

use crate::ui::style::MapStyle;
use qdo36_map::City;
use qdo36_tangibles::MapScale;
use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::DisplayRotation;

/// Persisted app-level settings (daemon endpoint, city board, locale).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// tangible-objects daemon GraphQL endpoint.
    pub daemon_url: String,
    /// Locale override; `None` = negotiate from the environment.
    pub locale: Option<String>,
    /// City board to play on.
    #[serde(default)]
    pub city: City,
    /// Base zoom of the map viewport.
    #[serde(default)]
    pub map_scale: MapScale,
    /// Display rotation: quarter turns applied to surface positions
    /// before the px scale (rotated desktop panels).
    #[serde(default)]
    pub display_rotation: DisplayRotation,
    /// The map render style.
    #[serde(default)]
    pub map_style: MapStyle,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            daemon_url: "http://127.0.0.1:8080".to_string(),
            locale: None,
            city: City::default(),
            map_scale: MapScale::default(),
            display_rotation: DisplayRotation::default(),
            map_style: MapStyle::default(),
        }
    }
}

impl AppSettings {
    /// Settings file path below the user config dir.
    fn path() -> PathBuf {
        glib::user_config_dir().join("qdo36").join("settings.toml")
    }

    /// Loads settings; falls back to defaults on any error.
    pub fn load() -> Self {
        let path = Self::path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&text).unwrap_or_default()
    }

    /// Persists settings, creating the directory when needed.
    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, text);
        }
    }
}
