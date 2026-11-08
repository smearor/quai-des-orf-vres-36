//! `widgets` - the shared building blocks the menu sections use.

use crate::l10n::L10n;

use crate::ui::map_scene::controls::mode_option::ModeOption;
use gtk4::Align;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Removes all children of a box (the drawer's single section).
pub(crate) fn clear(box_: &gtk4::Box) {
    while let Some(child) = box_.first_child() {
        box_.remove(&child);
    }
}

/// A bar toggle opening its section inside the inline `drawer`
/// row - unlike a popover, the row lives in the rotated widget
/// tree, so it faces the edge's player too. `build` constructs a
/// fresh section on each open so its controls mirror live state.
pub(crate) fn drawer_button(
    icon: &str,
    tip: &str,
    drawer: &gtk4::Box,
    open_toggle: &Rc<RefCell<Option<gtk4::ToggleButton>>>,
    build: impl Fn() -> gtk4::Widget + 'static,
) -> gtk4::ToggleButton {
    let button = gtk4::ToggleButton::builder().icon_name(icon).build();
    if !tip.is_empty() {
        button.set_tooltip_text(Some(tip));
    }
    let drawer = drawer.clone();
    let open_toggle = open_toggle.clone();
    button.connect_toggled(move |button| {
        if button.is_active() {
            // Close the previously open section's toggle first -
            // the borrow must end before `set_active` re-enters
            // this handler.
            let prev = open_toggle.borrow_mut().replace(button.clone());
            if let Some(prev) = prev
                && prev != *button
            {
                prev.set_active(false);
            }
            clear(&drawer);
            drawer.append(&build());
            drawer.set_visible(true);
        } else if open_toggle.borrow().as_ref() == Some(button) {
            open_toggle.borrow_mut().take();
            clear(&drawer);
            drawer.set_visible(false);
        }
    });
    button
}

/// A section caption inside an overlay popover.
pub(super) fn menu_heading(text: String) -> gtk4::Label {
    gtk4::Label::builder()
        .label(text)
        .halign(Align::Start)
        .css_classes(["heading"])
        .margin_top(4)
        .build()
}

/// Appends a radio row per option; `current` picks the initial
/// selection and `apply` publishes the picked mode.
pub(super) fn mode_radios<T: Copy + PartialEq + 'static>(list: &gtk4::Box, l10n: &L10n, current: T, options: &[ModeOption<T>], apply: fn(T)) {
    let mut group: Option<gtk4::CheckButton> = None;
    for option in options {
        let mode = option.mode;
        let check = gtk4::CheckButton::with_label(&l10n.msg_key(option.label));
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

/// A text row button inside an overlay popover menu.
pub(super) fn action_row(text: &str) -> gtk4::Button {
    let label = gtk4::Label::builder().label(text).halign(Align::Start).build();
    gtk4::Button::builder().child(&label).has_frame(false).build()
}
