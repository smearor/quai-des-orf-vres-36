//! `controls` - the on-screen control bar (zoom, layer toggles,
//! orientation modes, quit) floating at the screen edge over the
//! map scene, rotated toward that edge by `smearor-wrot-rotation`.

pub mod mode_option;
pub mod sections;
pub mod widgets;

use crate::l10n::L10n;
use crate::l10n::LabelKey;
use crate::mcp_bridge;
use crate::settings::AppSettings;
use crate::ui::map_scene::controls::sections::layers::layer_list;
use crate::ui::map_scene::controls::sections::orientation::orientation_list;
use crate::ui::map_scene::controls::sections::style::style_list;
use crate::ui::map_scene::controls::sections::tools::tools_list;
use crate::ui::map_scene::controls::widgets::clear;
use crate::ui::map_scene::controls::widgets::drawer_button;
use gtk4::Align;
use gtk4::Orientation;
use gtk4::prelude::*;
use smearor_wrot_rotation::RotationWidget;
use smearor_wrot_rotation::SmearorRotation;
use std::cell::RefCell;
use std::rc::Rc;

/// The menu's inset from the screen edge (px).
const MENU_INSET: i32 = 28;

/// How much room the scene reserves along the bottom edge for the
/// menu strip - the anchor hint floats above it (px).
pub const EDGE_RESERVE: f64 = 100.0;

/// One screen edge's menu placement: rotation toward the player
/// seated there plus the overlay alignment and margin side.
struct Side {
    /// Degrees the menu rotates so its text faces the edge
    /// (`smearor-wrot-rotation` convention: text-up points away
    /// from the player).
    degrees: f32,
    /// Horizontal anchor.
    halign: Align,
    /// Vertical anchor.
    valign: Align,
}

/// The four tabletop seats: bottom/top/left/right edge - each
/// menu reads toward its own player.
const SIDES: [Side; 4] = [
    Side {
        degrees: 0.0,
        halign: Align::Center,
        valign: Align::End,
    },
    Side {
        degrees: 180.0,
        halign: Align::Center,
        valign: Align::Start,
    },
    Side {
        degrees: 90.0,
        halign: Align::Start,
        valign: Align::Center,
    },
    Side {
        degrees: 270.0,
        halign: Align::End,
        valign: Align::Center,
    },
];

/// Builds one collapsible control menu per screen edge - each
/// wrapped in a `RotationWidget` facing its own player, and each
/// collapsed to a burger button until its side opens it (per-menu
/// state). The handlers mutate the scene through the same
/// `mcp_bridge` helpers the MCP commands use - always in sync.
pub fn controls(l10n: &Rc<L10n>, settings: &AppSettings) -> Vec<RotationWidget> {
    mcp_bridge::register_settings(settings);
    SIDES.iter().map(|side| side_menu(l10n, settings, side)).collect()
}

/// One edge's menu: burger button plus the collapsible bar inside
/// a `RotationWidget` rotated toward the edge's player. The menu
/// sections open inline below the bar (a drawer row) instead of a
/// popover: GTK popovers are separate surfaces that cannot
/// inherit the widget's rotation.
fn side_menu(l10n: &Rc<L10n>, settings: &AppSettings, side: &Side) -> RotationWidget {
    let column = gtk4::Box::builder().orientation(Orientation::Vertical).spacing(8).build();

    // The open section's content row - sits above the bar so it
    // expands away from the screen edge.
    let drawer = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .visible(false)
        .css_classes(["map-controls", settings.map_style.controls_class()])
        .build();
    column.append(&drawer);
    // At most one section is open at a time; the toggle it belongs
    // to is tracked here.
    let open_toggle: Rc<RefCell<Option<gtk4::ToggleButton>>> = Rc::new(RefCell::new(None));

    // The collapsible bar is a horizontal row sitting above the
    // burger so the button stays at the screen edge in every
    // rotation.
    let bar = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["map-controls", settings.map_style.controls_class()])
        .visible(false)
        .build();
    column.append(&bar);
    let burger = gtk4::Button::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text(l10n.msg_key(LabelKey("controls-menu")))
        .css_classes(["map-controls", settings.map_style.controls_class()])
        .build();
    {
        let bar = bar.clone();
        let drawer = drawer.clone();
        let open_toggle = open_toggle.clone();
        burger.connect_clicked(move |button| {
            let open = !bar.is_visible();
            bar.set_visible(open);
            drawer.set_visible(open && drawer.first_child().is_some());
            // Collapsing the menu also closes the open section -
            // the borrow must end before `set_active` re-enters
            // the toggle's handler.
            if !open {
                let open_section = open_toggle.borrow_mut().take();
                if let Some(toggle) = open_section {
                    toggle.set_active(false);
                }
                clear(&drawer);
            }
            button.set_icon_name(if open { "window-close-symbolic" } else { "open-menu-symbolic" });
        });
    }
    column.append(&burger);

    let zoom_in = gtk4::Button::from_icon_name("zoom-in-symbolic");
    zoom_in.connect_clicked(|_| mcp_bridge::zoom_by(1.25));
    bar.append(&zoom_in);

    let zoom_reset = gtk4::Button::from_icon_name("zoom-original-symbolic");
    zoom_reset.connect_clicked(|_| mcp_bridge::reset_view());
    bar.append(&zoom_reset);

    let zoom_out = gtk4::Button::from_icon_name("zoom-out-symbolic");
    zoom_out.connect_clicked(|_| mcp_bridge::zoom_by(0.8));
    bar.append(&zoom_out);

    {
        let l10n = l10n.clone();
        bar.append(&drawer_button(
            "object-rotate-right-symbolic",
            &l10n.msg_key(LabelKey("controls-orientation")),
            &drawer,
            &open_toggle,
            move || orientation_list(&l10n).upcast(),
        ));
    }
    {
        let l10n = l10n.clone();
        bar.append(&drawer_button(
            "view-list-symbolic",
            &l10n.msg_key(LabelKey("controls-layers")),
            &drawer,
            &open_toggle,
            move || layer_list(&l10n),
        ));
    }
    bar.append(&drawer_button(
        "preferences-color-symbolic",
        &l10n.msg_key(LabelKey("controls-style")),
        &drawer,
        &open_toggle,
        style_list,
    ));
    {
        let l10n = l10n.clone();
        bar.append(&drawer_button(
            "applications-engineering-symbolic",
            &l10n.msg_key(LabelKey("controls-tools")),
            &drawer,
            &open_toggle,
            move || tools_list(&l10n).upcast(),
        ));
    }

    bar.append(&gtk4::Separator::new(Orientation::Vertical));
    let quit = gtk4::Button::builder()
        .icon_name("system-shutdown-symbolic")
        .tooltip_text(l10n.msg_key(LabelKey("controls-quit")))
        .build();
    quit.connect_clicked(|_| mcp_bridge::quit_app());
    bar.append(&quit);

    let rotation = RotationWidget::new(SmearorRotation::new(side.degrees));
    rotation.set_child(Some(&column));
    rotation.set_halign(side.halign);
    rotation.set_valign(side.valign);
    rotation.set_margin_start(MENU_INSET);
    rotation.set_margin_end(MENU_INSET);
    rotation.set_margin_top(MENU_INSET);
    rotation.set_margin_bottom(MENU_INSET);
    mcp_bridge::register_menu(&[bar.clone().upcast(), drawer.clone().upcast(), burger.clone().upcast()]);
    rotation
}
