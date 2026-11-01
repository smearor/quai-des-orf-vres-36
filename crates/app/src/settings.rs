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

#[cfg(test)]
mod tests {
    use super::*;
    use qdo36_map::City;

    /// Fields missing from the file keep their defaults (file >
    /// default precedence relies on serde defaults).
    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let settings: AppSettings = toml::from_str("daemon_url = \"http://surface:4711\"\ncity = \"berlin\"").unwrap_or_default();
        assert_eq!(settings.daemon_url, "http://surface:4711");
        assert_eq!(settings.city, City::Berlin);
        assert_eq!(settings.locale, None);
        assert_eq!(settings.map_style, MapStyle::default());
    }

    /// A corrupt settings file does not fail startup - the defaults
    /// apply.
    #[test]
    fn invalid_toml_falls_back_to_defaults() {
        let settings: AppSettings = toml::from_str("= not toml").unwrap_or_default();
        assert_eq!(settings.daemon_url, AppSettings::default().daemon_url);
    }

    /// The settings round-trip through TOML losslessly.
    #[test]
    fn save_load_round_trip() {
        let settings = AppSettings {
            daemon_url: "http://surface:4711".to_string(),
            locale: Some("de".to_string()),
            city: City::Berlin,
            ..AppSettings::default()
        };
        let text = toml::to_string_pretty(&settings).unwrap_or_default();
        let restored: AppSettings = toml::from_str(&text).unwrap_or_default();
        assert_eq!(restored.daemon_url, settings.daemon_url);
        assert_eq!(restored.locale, settings.locale);
        assert_eq!(restored.city, settings.city);
        assert_eq!(restored.map_style, settings.map_style);
    }
}
