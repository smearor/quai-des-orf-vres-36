use std::rc::Rc;

use libadwaita as adw;
use libadwaita::prelude::*;

use crate::l10n::L10n;
use crate::settings::AppSettings;

/// The application window: a fullscreen stack switching between the
/// daemon setup screen and the map scene.
pub struct AppWindow {
    /// The GTK window.
    pub window: adw::ApplicationWindow,
}

impl AppWindow {
    /// Builds the window and its screens.
    pub fn new(app: &adw::Application, settings: AppSettings, l10n: Rc<L10n>) -> Self {
        let window = adw::ApplicationWindow::builder().application(app).build();

        let stack = gtk4::Stack::new();
        let setup = adw::StatusPage::builder()
            .title(l10n.msg("app-title"))
            .description(l10n.msg("daemon-connecting"))
            .icon_name("qdo-map-symbolic")
            .build();
        stack.add_named(&setup, Some("setup"));

        window.set_content(Some(&stack));
        let _ = settings;
        Self { window }
    }
}
