//! `LayerToggle` - one checkbox row of the overlay menu.

use crate::ui::map_scene::LayerName;

/// A named layer switch: the checkbox caption, the boxed mutator
/// applied on toggle and the initial checkbox state read from the
/// live `SceneLayers`.
pub struct LayerToggle {
    /// Checkbox caption (layer name).
    pub label: LayerName,
    /// The layer mutation applied when the checkbox toggles.
    pub toggle: Box<dyn Fn(bool)>,
    /// Initial checkbox state (current layer visibility).
    pub active: bool,
}
