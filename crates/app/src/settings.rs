use std::path::PathBuf;

use crate::Args;
use crate::heading_offset_deg::HeadingOffsetDeg;
use crate::layers_config::LayersConfig;
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
    /// Calibration offset between the Eiffel tower profile's pin
    /// frame and map north, in degrees (WP04 data model).
    #[serde(default)]
    pub heading_offset_deg: HeadingOffsetDeg,
    /// Persisted layer visibility + caption orientations
    /// (stations, landmarks, districts, parks).
    #[serde(default)]
    pub layers: LayersConfig,
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
            heading_offset_deg: HeadingOffsetDeg::default(),
            layers: LayersConfig::default(),
        }
    }
}

impl AppSettings {
    /// Settings file path below the user config dir.
    fn path() -> PathBuf {
        glib::user_config_dir().join("qdo36").join("config.toml")
    }

    /// Loads settings; falls back to defaults on any error.
    pub fn load() -> Self {
        let path = Self::path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&text).unwrap_or_default()
    }

    /// Applies the CLI overrides (sticky: they are saved afterwards
    /// and stay effective until changed again).
    pub fn apply_overrides(&mut self, args: &Args) {
        if let Some(daemon_url) = args.daemon_url.clone() {
            self.daemon_url = daemon_url;
        }
        if let Some(locale) = args.locale.clone() {
            self.locale = Some(locale);
        }
        if let Some(city) = args.city {
            self.city = city;
        }
        if let Some(rotation) = args.display_rotation {
            self.display_rotation = rotation;
        }
        if let Some(heading) = args.heading_offset {
            self.heading_offset_deg = heading;
        }
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
    use clap::Parser;
    use qdo36_map::City;

    /// CLI args parsed from the given flag list.
    fn args(flags: &[&str]) -> Args {
        let argv: Vec<&str> = ["qdo36"].into_iter().chain(flags.iter().copied()).collect();
        Args::parse_from(argv)
    }

    /// No flags: the persisted values pass through unchanged
    /// (file > default).
    #[test]
    fn omitted_flags_keep_the_persisted_values() {
        let mut settings = AppSettings {
            daemon_url: "http://saved:1".to_string(),
            city: City::Berlin,
            ..AppSettings::default()
        };
        settings.apply_overrides(&args(&[]));
        assert_eq!(settings.daemon_url, "http://saved:1");
        assert_eq!(settings.city, City::Berlin);
    }

    /// Passed flags override the persisted values (CLI > file).
    #[test]
    fn cli_flags_beat_the_persisted_values() {
        let mut settings = AppSettings {
            daemon_url: "http://saved:1".to_string(),
            city: City::Berlin,
            locale: Some("de".to_string()),
            ..AppSettings::default()
        };
        settings.apply_overrides(&args(&["--daemon-url", "http://cli:2", "--city", "paris", "--locale", "en"]));
        assert_eq!(settings.daemon_url, "http://cli:2");
        assert_eq!(settings.city, City::Paris);
        assert_eq!(settings.locale.as_deref(), Some("en"));
    }

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
