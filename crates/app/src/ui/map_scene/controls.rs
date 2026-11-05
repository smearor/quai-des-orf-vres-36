//! `controls` - the on-screen control bar (zoom, layer toggles,
//! orientation modes) floating at the screen edge over the map scene.

use crate::l10n::L10n;
use crate::mcp_bridge;
use crate::ui::map_scene::LabelMode;
use crate::ui::map_scene::LayerName;
use crate::ui::map_scene::LayerToggle;
use crate::ui::style::MapStyle;
use gtk4::Align;
use gtk4::Orientation;
use gtk4::prelude::*;
use std::rc::Rc;
use tracing::debug;

/// Builds the vertical control bar anchored to the screen edge.
/// The handlers mutate the scene through the same `mcp_bridge`
/// helpers the MCP commands use - always in sync.
pub fn controls(l10n: &Rc<L10n>) -> gtk4::Box {
    let bar = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["map-controls"])
        .halign(Align::End)
        .valign(Align::Center)
        .margin_end(12)
        .build();

    let zoom_in = gtk4::Button::from_icon_name("zoom-in-symbolic");
    zoom_in.connect_clicked(|_| mcp_bridge::zoom_by(1.25));
    bar.append(&zoom_in);

    let zoom_reset = gtk4::Button::from_icon_name("zoom-original-symbolic");
    zoom_reset.connect_clicked(|_| mcp_bridge::reset_view());
    bar.append(&zoom_reset);

    let zoom_out = gtk4::Button::from_icon_name("zoom-out-symbolic");
    zoom_out.connect_clicked(|_| mcp_bridge::zoom_by(0.8));
    bar.append(&zoom_out);

    bar.append(&orientation_menu(l10n));
    bar.append(&layer_menu());
    bar.append(&style_menu());
    bar.append(&tools_menu(l10n));
    bar
}

/// The orientation popover: a Radial/Screen radio pair per
/// rotatable overlay (station labels, landmark pins).
fn orientation_menu(l10n: &Rc<L10n>) -> gtk4::MenuButton {
    let button = gtk4::MenuButton::builder()
        .icon_name("object-rotate-right-symbolic")
        .tooltip_text(l10n.msg("controls-orientation"))
        .build();
    let popover = gtk4::Popover::new();
    popover.set_child(Some(&orientation_list(l10n)));
    {
        let l10n = l10n.clone();
        // Rebuilt on each popup: the radios always mirror the live
        // scene state even when MCP flips a mode meanwhile.
        popover.connect_map(move |popover| popover.set_child(Some(&orientation_list(&l10n))));
    }
    button.set_popover(Some(&popover));
    button
}

/// The orientation popover content - fresh radio states on every
/// (re)build.
fn orientation_list(l10n: &L10n) -> gtk4::Box {
    let list = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let layers = mcp_bridge::current_layers().unwrap_or_default();
    list.append(&menu_heading(l10n.msg("controls-labels")));
    mode_radios(&list, l10n, layers.label_mode, |mode| mcp_bridge::mutate_layers(move |layers| layers.label_mode = mode));
    list.append(&gtk4::Separator::new(Orientation::Horizontal));
    list.append(&menu_heading(l10n.msg("controls-landmarks")));
    mode_radios(&list, l10n, layers.landmark_mode, |mode| mcp_bridge::mutate_layers(move |layers| layers.landmark_mode = mode));
    list
}

/// A section caption inside an overlay popover.
fn menu_heading(text: String) -> gtk4::Label {
    gtk4::Label::builder()
        .label(text)
        .halign(Align::Start)
        .css_classes(["heading"])
        .margin_top(4)
        .build()
}

/// Appends the Radial/Screen radio pair; `current` picks the
/// initial selection and `apply` publishes the picked mode.
fn mode_radios(list: &gtk4::Box, l10n: &L10n, current: LabelMode, apply: fn(LabelMode)) {
    let mut group: Option<gtk4::CheckButton> = None;
    for (id, mode) in [("controls-mode-radial", LabelMode::Radial), ("controls-mode-screen", LabelMode::Screen)] {
        let check = gtk4::CheckButton::with_label(&l10n.msg(id));
        check.set_active(current == mode);
        if let Some(first) = &group {
            check.set_group(Some(first));
        } else {
            group = Some(check.clone());
        }
        check.connect_toggled(move |check| {
            if check.is_active() {
                apply(mode);
            }
        });
        list.append(&check);
    }
}

/// The tools popover: screenshot actions - the full window straight
/// away, or an armed drag for the area selection.
fn tools_menu(l10n: &L10n) -> gtk4::MenuButton {
    let button = gtk4::MenuButton::builder()
        .icon_name("applications-engineering-symbolic")
        .tooltip_text(l10n.msg("controls-tools"))
        .build();
    let popover = gtk4::Popover::new();
    let list = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let shot_all = action_row(&l10n.msg("controls-shot-all"));
    {
        let popover = popover.clone();
        shot_all.connect_clicked(move |_| {
            debug!("tools menu: take-screenshot clicked");
            popover.popdown();
            // Schedules on after-paint: the popover teardown just
            // invalidated the window's cached render node.
            mcp_bridge::save_shot(None);
        });
    }
    list.append(&shot_all);
    let shot_area = action_row(&l10n.msg("controls-shot-area"));
    {
        let popover = popover.clone();
        shot_area.connect_clicked(move |_| {
            debug!("tools menu: select-area clicked, arming");
            popover.popdown();
            mcp_bridge::arm_area_shot();
        });
    }
    list.append(&shot_area);
    popover.set_child(Some(&list));
    button.set_popover(Some(&popover));
    button
}

/// A text row button inside an overlay popover menu.
fn action_row(text: &str) -> gtk4::Button {
    let label = gtk4::Label::builder().label(text).halign(Align::Start).build();
    gtk4::Button::builder().child(&label).has_frame(false).build()
}

/// The style popover: radio picks per artwork style, live via the
/// same bridge the `set_style` MCP command uses.
fn style_menu() -> gtk4::MenuButton {
    let button = gtk4::MenuButton::builder().icon_name("preferences-color-symbolic").tooltip_text("Stil").build();
    let popover = gtk4::Popover::new();
    let list = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let styles = [("Comic", MapStyle::Comic), ("Belle Époque", MapStyle::BelleEpoque), ("Neon", MapStyle::Neon)];
    let current = mcp_bridge::current_style().unwrap_or_default();
    let mut group: Option<gtk4::CheckButton> = None;
    for (label, style) in styles {
        let check = gtk4::CheckButton::with_label(label);
        check.set_active(style == current);
        if let Some(first) = &group {
            check.set_group(Some(first));
        } else {
            group = Some(check.clone());
        }
        check.connect_toggled(move |check| {
            if check.is_active() {
                mcp_bridge::set_map_style(style);
            }
        });
        list.append(&check);
    }
    popover.set_child(Some(&list));
    button.set_popover(Some(&popover));
    button
}

/// The overlay popover: checkboxes per scene layer.
fn layer_menu() -> gtk4::MenuButton {
    let button = gtk4::MenuButton::builder().icon_name("view-list-symbolic").build();
    let popover = gtk4::Popover::new();
    let list = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let initial = mcp_bridge::current_layers().unwrap_or_default();
    for row in layer_toggles(&initial) {
        let check = gtk4::CheckButton::with_label(row.label.as_str());
        check.set_active(row.active);
        check.connect_toggled(move |check| (row.toggle)(check.is_active()));
        list.append(&check);
    }
    popover.set_child(Some(&list));
    button.set_popover(Some(&popover));
    button
}

/// Layer-toggle closures + the initial checkbox state, boxed so
/// the checkbox list stays generic.
fn layer_toggles(initial: &crate::ui::map_scene::SceneLayers) -> Vec<LayerToggle> {
    LayerName::ALL
        .iter()
        .map(|&name| LayerToggle {
            label: name,
            toggle: Box::new(move |on| mcp_bridge::mutate_layers(move |l| l.set(name, on))),
            active: initial.get(name),
        })
        .collect()
}
