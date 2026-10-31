use crate::AnchorSpec;
use crate::CityName;
use crate::MapSlogan;
use crate::MapStory;
use qdo36_material::RoundNumber;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;
use tangible_objects_model::Region;
use tangible_objects_model::serde_quad;

/// Map-wide metadata of the dataset.
///
/// Carries the city's identity and rule defaults: Paris ships the
/// Scotland Yard classics (24 rounds, reveals at 3/8/13/18/24); other
/// cities may tune both - different street geometry justifies
/// different pacing.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MapMeta {
    /// The city this dataset describes ("paris", "berlin").
    pub name: CityName,
    /// One-line city flavor for the splash/setup screens.
    #[serde(default)]
    pub slogan: MapSlogan,
    /// Short noir framing text (2-3 sentences) for the setup dialog.
    #[serde(default)]
    pub story: MapStory,
    /// Map coordinate bounds, TOML shape `bounds_mm = [x_min, y_min, x_max, y_max]`.
    #[serde(rename = "bounds_mm", with = "serde_quad")]
    pub bounds: Region,
    /// Maximum number of rounds; the fugitive wins when reached.
    ///
    /// Default: the Scotland Yard classic, 24.
    #[serde(default = "default_max_rounds")]
    pub max_rounds: RoundNumber,
    /// Rounds at which the fugitive's position is revealed.
    ///
    /// Default: the classic schedule `[3, 8, 13, 18, 24]`.
    #[serde(default = "default_reveal_rounds")]
    pub reveal_rounds: BTreeSet<RoundNumber>,
    /// The city's anchor landmark and the station it pins down.
    ///
    /// `None` for generated maps without a designated anchor
    /// (`qdo36-mapgen` output until curated).
    #[serde(default)]
    pub anchor: Option<AnchorSpec>,
}

fn default_max_rounds() -> RoundNumber {
    RoundNumber::DEFAULT_MAX
}

fn default_reveal_rounds() -> BTreeSet<RoundNumber> {
    [3, 8, 13, 18, 24].into_iter().map(RoundNumber::new).collect()
}
