//! `orientation` - the orientation drawer's Radial/Screen radios.

use crate::l10n::HasLabelKey;
use crate::l10n::L10n;
use crate::l10n::LabelKey;
use crate::mcp_bridge;
use crate::ui::map_scene::LabelMode;
use crate::ui::map_scene::controls::mode_option::ModeOption;
use crate::ui::map_scene::controls::widgets::menu_heading;
use crate::ui::map_scene::controls::widgets::mode_radios;
use crate::ui::style::HouseMode;
use gtk4::Orientation;
use gtk4::prelude::*;

/// One rotatable overlay's orientation group: heading, the live
/// mode and the apply callback.
type OrientationGroup = (String, LabelMode, fn(LabelMode));

impl HasLabelKey for HouseMode {
    fn label_key(&self) -> LabelKey {
        LabelKey(match self {
            HouseMode::RadialIso => "controls-mode-radial-iso",
            HouseMode::Iso => "controls-mode-iso",
            HouseMode::TopDown => "controls-mode-topdown",
        })
    }
}

/// The house-mode radio options (i18n key + mode).
fn house_options() -> [ModeOption<HouseMode>; 3] {
    HouseMode::ALL.map(|mode| ModeOption { label: mode.label_key(), mode })
}

/// The orientation drawer content: a Radial/Screen radio pair per
/// rotatable overlay - fresh states on every build.
pub(crate) fn orientation_list(l10n: &L10n) -> gtk4::Box {
    let list = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let layers = mcp_bridge::current_layers().unwrap_or_default();
    let label_options: &[ModeOption<LabelMode>] = &[
        ModeOption {
            label: LabelKey("controls-mode-radial"),
            mode: LabelMode::Radial,
        },
        ModeOption {
            label: LabelKey("controls-mode-screen"),
            mode: LabelMode::Screen,
        },
    ];
    // A (heading, current mode, apply) row per rotatable overlay.
    let groups: [OrientationGroup; 4] = [
        (l10n.msg_key(LabelKey("controls-labels")), layers.label_mode, |mode| {
            mcp_bridge::mutate_layers(move |layers| layers.label_mode = mode)
        }),
        (l10n.msg_key(LabelKey("controls-landmarks")), layers.landmark_mode, |mode| {
            mcp_bridge::mutate_layers(move |layers| layers.landmark_mode = mode)
        }),
        (l10n.msg_key(LabelKey("controls-districts")), layers.district_mode, |mode| {
            mcp_bridge::mutate_layers(move |layers| layers.district_mode = mode)
        }),
        (l10n.msg_key(LabelKey("controls-parks")), layers.park_mode, |mode| {
            mcp_bridge::mutate_layers(move |layers| layers.park_mode = mode)
        }),
    ];
    for (heading, current, apply) in groups.into_iter() {
        let group = gtk4::Box::builder().orientation(Orientation::Vertical).spacing(4).build();
        group.append(&menu_heading(heading));
        mode_radios(&group, l10n, current, label_options, apply);
        list.append(&group);
    }
    // The houses pick a rendering mode instead - three options,
    // same radio-row widget.
    let group = gtk4::Box::builder().orientation(Orientation::Vertical).spacing(4).build();
    group.append(&menu_heading(l10n.msg_key(LabelKey("controls-houses"))));
    mode_radios(&group, l10n, layers.house_mode, &house_options(), |mode| {
        mcp_bridge::mutate_layers(move |layers| layers.house_mode = mode)
    });
    list.append(&group);
    list
}
