//! `controls` - the on-screen control bar (zoom, layer toggles,
//! label mode) floating at the screen edge over the map scene.

use crate::mcp_bridge;
use crate::ui::map_scene::LayerName;
use crate::ui::map_scene::LayerToggle;
use gtk4::Align;
use gtk4::Orientation;
use gtk4::prelude::*;
use crate::ui::style::MapStyle;

/// Builds the vertical control bar anchored to the screen edge.
/// The handlers mutate the scene through the same `mcp_bridge`
/// helpers the MCP commands use - always in sync.
pub fn controls() -> gtk4::Box {
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

    let label_mode = gtk4::Button::from_icon_name("object-rotate-right-symbolic");
    label_mode.set_tooltip_text(Some("labels: radial / screen"));
    label_mode.connect_clicked(|_| mcp_bridge::toggle_label_mode());
    bar.append(&label_mode);

    bar.append(&layer_menu());
    bar.append(&style_menu());
    bar
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
    let styles = [
        ("Comic", MapStyle::Comic),
        ("Belle Époque", MapStyle::BelleEpoque),
        ("Neon", MapStyle::Neon),
    ];
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
