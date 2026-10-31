//! Custom icon set for Quai des Orfèvres 36.
//!
//! Built at compile time from `icons.toml` via
//! [`fonts_rs_generator::IconSetBuild`]: a whitelist of SVGs from the
//! fonts-rs family crates merged into one GResource bundle with a
//! type-safe `icon_names` API.
//!
//! ```rust,ignore
//! qdo36_icons::Qdo36Icons::install().expect("icon set registration failed");
//! let image = gtk4::Image::from_icon_name(qdo36_icons::icon_names::QDO_MAP_SYMBOLIC);
//! ```

pub mod icons {
    include!(concat!(env!("OUT_DIR"), "/iconset.rs"));
}

pub use icons::*;
