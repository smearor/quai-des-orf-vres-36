//! `tools` - the tools drawer's screenshot actions.

use crate::l10n::L10n;
use crate::l10n::LabelKey;
use crate::mcp_bridge;
use crate::ui::map_scene::controls::widgets::action_row;
use gtk4::Orientation;
use gtk4::prelude::*;
use tracing::debug;

/// The tools drawer content: screenshot actions - the full window
/// straight away, or an armed drag for the area selection.
pub(crate) fn tools_list(l10n: &L10n) -> gtk4::Box {
    let list = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let shot_all = action_row(&l10n.msg_key(LabelKey("controls-shot-all")));
    shot_all.connect_clicked(move |_| {
        debug!("tools menu: take-screenshot clicked");
        // Schedules on after-paint: the menu toggle just
        // invalidated the window's cached render node.
        mcp_bridge::save_shot(None);
    });
    list.append(&shot_all);
    let shot_area = action_row(&l10n.msg_key(LabelKey("controls-shot-area")));
    shot_area.connect_clicked(move |_| {
        debug!("tools menu: select-area clicked, arming");
        mcp_bridge::arm_area_shot();
    });
    list.append(&shot_area);
    list
}
