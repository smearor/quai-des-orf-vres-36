use crate::AnchorSpec;
use crate::CityName;
use crate::FugitiveSpec;
use crate::LineColor;
use crate::LineName;
use crate::MapSlogan;
use crate::MapStory;
use crate::Transport;
use crate::TransportName;
use qdo36_material::RoundNumber;
use qdo36_material::TicketCounts;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
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
    /// The city's fugitive: alias + home station (`[map.fugitive]`).
    ///
    /// `None` for generated maps without a designated fugitive.
    #[serde(default)]
    pub fugitive: Option<FugitiveSpec>,
    /// The city's own transport names
    /// (`[map.transport_names]` - batobus becomes "Spreefähre" in
    /// Berlin, "Batobus" in Paris).
    #[serde(default)]
    pub transport_names: BTreeMap<Transport, TransportName>,
    /// Transit lines with brand colors
    /// (`[map.lines]` - "U1" = "#7DAD4C" in Berlin, "M1" = "#FFCD00" in
    /// Paris); referenced by metro edges.
    #[serde(default)]
    pub lines: BTreeMap<LineName, LineColor>,
    /// Map coordinate bounds, TOML shape `bounds_mm = [x_min, y_min, x_max, y_max]`.
    #[serde(rename = "bounds_mm", with = "serde_quad")]
    pub bounds: Region,
    /// Maximum number of rounds; the fugitive wins when reached.
    ///
    /// Default: the Scotland Yard classic, 24.
    #[serde(default = "default_max_rounds")]
    pub max_rounds: RoundNumber,
    /// The city's detective starting stock (`[map]
    /// detective_stock`); defaults to the classic 10 taxi / 8 bus /
    /// 4 metro.
    #[serde(default = "TicketCounts::detective_start")]
    pub detective_stock: TicketCounts,
    /// The city's fugitive starting pool (`[map] fugitive_stock`);
    /// defaults to unlimited travel + 5 black + 2 double.
    #[serde(default = "TicketCounts::fugitive_start")]
    pub fugitive_stock: TicketCounts,
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
