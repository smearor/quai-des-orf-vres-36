//! Quai des Orfèvres 36 - Scotland Yard tabletop game for the smart desk.

mod anchor_tracker;
mod controller;
mod fonts;
mod game_host;
mod heading_offset_deg;
mod l10n;
mod mcp_bridge;
mod settings;
mod tangible_host;
mod ui;

use crate::heading_offset_deg::HeadingOffsetDeg;
use crate::l10n::L10n;
use crate::settings::AppSettings;
use crate::ui::app_window::AppWindow;
use glib::MainContext;
use gtk4::prelude::*;
use libadwaita as adw;
use qdo36_icons::Qdo36Icons;
use qdo36_map::City;
use signal_hook::iterator::Signals;
use std::net::SocketAddr;
use std::rc::Rc;
use tangible_objects_model::DisplayRotation;
use tracing::warn;
use tracing_subscriber::EnvFilter;
use unic_langid::LanguageIdentifier;

/// Command-line options.
#[derive(Debug, clap::Parser)]
#[command(name = "qdo36", about = "Quai des Orfèvres 36 - Scotland Yard for the smart desk")]
struct Args {
    /// tangible-objects daemon GraphQL endpoint.
    #[arg(long)]
    daemon_url: Option<String>,
    /// Locale override (e.g. `de`, `en`).
    #[arg(long)]
    locale: Option<String>,
    /// City board (`paris` or `berlin`).
    #[arg(long)]
    city: Option<City>,
    /// Expose the game-domain MCP endpoint (Streamable HTTP, e.g.
    /// `127.0.0.1:8091` for LAN agents) - reserved for WP05.
    #[arg(long)]
    mcp_listen: Option<SocketAddr>,
    /// Display rotation: quarter turns applied to surface-mm
    /// positions before the mm-to-pixel scale - needed when the
    /// desktop shows the panel rotated (e.g. GNOME portrait on a
    /// landscape touchscreen).
    #[arg(long, value_name = "0|90|180|270")]
    display_rotation: Option<DisplayRotation>,
    /// Calibration offset between the Eiffel tower's pin frame and
    /// map north, in degrees.
    #[arg(long, value_name = "DEG")]
    heading_offset: Option<HeadingOffsetDeg>,
}

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let args = <Args as clap::Parser>::parse();
    let mut settings = AppSettings::load();
    settings.apply_overrides(&args);
    // CLI flags are sticky overrides: they land in the persisted
    // settings and stay effective until changed again.
    settings.save();

    let l10n = settings
        .locale
        .as_deref()
        .and_then(|code| code.parse::<LanguageIdentifier>().ok())
        .map_or_else(L10n::system, L10n::for_locale);
    let l10n = Rc::new(l10n);

    let application = adw::Application::builder().application_id("io.github.smearor.qdo36").build();

    // Bundled OFL fonts must be registered before the first text layout,
    // so they land here instead of inside `connect_activate`.
    fonts::install();

    // The MCP endpoint shares state through a watch snapshot; without
    // the flag the app runs unchanged.
    let mcp_listen = args.mcp_listen;

    application.connect_activate(move |app| {
        if let Some(addr) = mcp_listen {
            mcp_bridge::attach(addr);
        }
        if let Err(err) = Qdo36Icons::install() {
            warn!("icon set install failed: {err}");
        }
        if let Some(display) = gdk4::Display::default()
            && let Err(err) = Qdo36Icons::install_icon_theme(&display)
        {
            warn!("icon theme registration failed: {err}");
        }
        install_css();
        install_shutdown_handlers(app);
        let window = AppWindow::new(app, settings.clone(), l10n.clone());
        window.window.present();
        if std::env::var_os("QDO36_NO_FULLSCREEN").is_none() {
            // Fullscreen must be requested once the window is
            // presented; before that the content's natural size can
            // overshoot the monitor and the compositor clamps it
            // visually.
            window.window.fullscreen();
        }
    });

    // `run()` would feed argv to GTK's own option parser, which rejects
    // our flags; clap already consumed them.
    application.run_with_args(&["qdo36"])
}

/// Quits the app on SIGINT/SIGTERM so Ctrl+C on the desk exits
/// cleanly. Daemon registration cleanup (`unregister_owner`) hooks in
/// here once the controller exists (P2).
fn install_shutdown_handlers(app: &adw::Application) {
    let (tx, rx) = async_channel::unbounded::<()>();
    std::thread::spawn(move || {
        let mut signals = match Signals::new([signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM]) {
            Ok(signals) => signals,
            Err(err) => {
                warn!("signal handler install failed: {err}");
                return;
            }
        };
        if signals.forever().next().is_some() {
            let _ = tx.send_blocking(());
        }
    });
    let app = app.clone();
    MainContext::default().spawn_local(async move {
        if rx.recv().await.is_ok() {
            controller::with_controller(|controller| {
                controller.borrow_mut().release_registrations();
            });
            app.quit();
        }
    });
}

/// Registers the application stylesheet.
fn install_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(include_str!("../resources/style.css"));
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}
