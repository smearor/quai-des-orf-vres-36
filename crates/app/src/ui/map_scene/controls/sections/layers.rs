//! `layers` - the layer drawer's grouped checkboxes.

use crate::l10n::HasLabelKey;
use crate::l10n::L10n;
use crate::mcp_bridge;
use crate::ui::map_scene::LayerName;
use crate::ui::map_scene::LayerToggle;
use crate::ui::map_scene::SceneLayers;
use crate::ui::map_scene::controls::sections::layer_group::LayerGroup;
use crate::ui::map_scene::controls::widgets::menu_heading;
use gtk4::Orientation;
use gtk4::prelude::*;
use std::collections::HashMap;

/// The layer drawer content: checkboxes per scene layer, grouped
/// by topic so the row stays scannable.
pub(crate) fn layer_list(l10n: &L10n) -> gtk4::Widget {
    let list = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let initial = mcp_bridge::current_layers().unwrap_or_default();
    let mut toggles: HashMap<LayerName, LayerToggle> = layer_toggles(&initial).into_iter().map(|row| (row.label, row)).collect();
    for group in LayerGroup::ALL {
        let column = gtk4::Box::builder().orientation(Orientation::Vertical).spacing(4).build();
        column.append(&menu_heading(l10n.msg_key(group.label_key())));
        for &name in group.layer_names() {
            let Some(row) = toggles.remove(&name) else { continue };
            let check = gtk4::CheckButton::with_label(row.label.as_str());
            check.set_active(row.active);
            check.connect_toggled(move |check| (row.toggle)(check.is_active()));
            column.append(&check);
        }
        list.append(&column);
    }
    list.upcast()
}

/// Layer-toggle closures + the initial checkbox state, boxed so
/// the checkbox list stays generic.
fn layer_toggles(initial: &SceneLayers) -> Vec<LayerToggle> {
    LayerName::ALL
        .iter()
        .map(|&name| LayerToggle {
            label: name,
            toggle: Box::new(move |on| mcp_bridge::mutate_layers(move |layers| layers.set(name, on))),
            active: initial.get(name),
        })
        .collect()
}
