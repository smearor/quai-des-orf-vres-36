use crate::DaemonError;
use qdo36_map::City;
use tangible_objects_model::ProfileFile;

/// The city's anchor landmark pattern.
///
/// Bundles the per-city anchor profiles
/// (`data/<city>/<landmark>.toml` - Paris: Tour Eiffel, Berlin:
/// Fernsehturm) so the app can `registerPattern` them without file
/// I/O. Rotation stability comes from the daemon's
/// correspondence-anchored pin map (tangible-objects-rs >= bc109b8) - no
/// client-side solving needed.
#[derive(Debug, Clone)]
pub struct AnchorPattern {
    profile: ProfileFile,
}

impl AnchorPattern {
    /// The bundled anchor profile of a city.
    ///
    /// # Errors
    ///
    /// Returns [`DaemonError::Profile`] when the embedded TOML is
    /// invalid (a broken build artifact - the datasets are validated
    /// by tests).
    pub fn for_city(city: City) -> Result<Self, DaemonError> {
        match city {
            City::Paris => Self::from_toml(include_str!("../../data/paris/tour-de-eifel.toml")),
            City::Berlin => Self::from_toml(include_str!("../../data/berlin/fernsehturm.toml")),
        }
    }

    /// Parses a profile from TOML.
    ///
    /// # Errors
    ///
    /// Returns [`DaemonError::Profile`] on malformed TOML.
    pub fn from_toml(source: &str) -> Result<Self, DaemonError> {
        let profile: ProfileFile = toml::from_str(source).map_err(|e| DaemonError::Profile(e.to_string()))?;
        Ok(Self { profile })
    }

    /// The underlying profile file for `register_pattern`.
    pub fn profile(&self) -> &ProfileFile {
        &self.profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_city_profiles_parse() {
        for city in City::ALL {
            let pattern = AnchorPattern::for_city(city).unwrap_or_else(|e| panic!("{city}: {e}"));
            assert_eq!(pattern.profile().num_points, 4, "{city}: the anchor landmark has 4 pins");
        }
    }
}
