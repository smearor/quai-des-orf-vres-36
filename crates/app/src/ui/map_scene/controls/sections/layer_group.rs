//! `LayerGroup` - one checkbox group in the layer drawer.

use crate::l10n::HasLabelKey;
use crate::l10n::LabelKey;
use crate::ui::map_scene::LayerName;

/// One checkbox group in the layer drawer: `layer_names()` lists
/// the group's checkboxes, the i18n heading key resolves through
/// `HasLabelKey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerGroup {
    /// Transport edges + overlay extras.
    Transports,
    /// Backdrop: water, blocks, landmarks.
    Map,
    /// Station discs + captions.
    Stations,
    /// Extras: underground, compass.
    Extras,
}

impl LayerGroup {
    /// Every group, left to right in the drawer.
    pub const ALL: [LayerGroup; 4] = [Self::Transports, Self::Map, Self::Stations, Self::Extras];

    /// The layers the group lists.
    pub const fn layer_names(self) -> &'static [LayerName] {
        match self {
            Self::Transports => &[
                LayerName::Taxi,
                LayerName::Bus,
                LayerName::Metro,
                LayerName::MetroOverlay,
                LayerName::Batobus,
                LayerName::ParallelTransports,
            ],
            Self::Map => &[LayerName::Water, LayerName::Blocks, LayerName::Landmarks],
            Self::Stations => &[LayerName::Stations, LayerName::Labels, LayerName::Numbers, LayerName::Scores],
            Self::Extras => &[LayerName::Underground, LayerName::Compass],
        }
    }
}

impl HasLabelKey for LayerGroup {
    fn label_key(&self) -> LabelKey {
        LabelKey(match self {
            Self::Transports => "controls-group-transports",
            Self::Map => "controls-group-map",
            Self::Stations => "controls-group-stations",
            Self::Extras => "controls-group-extras",
        })
    }
}
