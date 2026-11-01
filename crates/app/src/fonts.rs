//! Bundled font registration for the GTK frontend.
//!
//! The OFL TTFs shipped in `resources/fonts` are registered with the
//! process fontconfig at startup, so Pango text (Fluent labels, HUD,
//! overlays) resolves the bundled families (Patrick Hand, Comic Neue,
//! Cormorant Garamond, EB Garamond, Courier Prime, Orbitron, Share Tech
//! Mono) instead of relying on whatever the host system happens to
//! install. Generated artwork needs no fonts at all — its `<text>`
//! elements are already outlined to paths by `qdo36-artwork`.

use fontconfig_sys as fc;
use std::ffi::CString;
use std::path::PathBuf;
use tracing::info;
use tracing::warn;

/// Registers the bundled font directory with fontconfig.
///
/// Runs before the application activates so the faces are available to
/// the first text layout. A missing directory is non-fatal: the app
/// keeps working with system-font fallbacks.
pub(crate) fn install() {
    let fonts_dir = bundled_fonts_dir();
    if !fonts_dir.is_dir() {
        warn!(dir = %fonts_dir.display(), "bundled font directory missing; falling back to system fonts");
        return;
    }
    let Ok(path) = CString::new(fonts_dir.as_os_str().to_string_lossy().as_bytes()) else {
        warn!("bundled font path is not valid UTF-8; falling back to system fonts");
        return;
    };
    unsafe {
        let config = fc::FcConfigGetCurrent();
        if config.is_null() {
            warn!("fontconfig unavailable; falling back to system fonts");
            return;
        }
        if fc::FcConfigAppFontAddDir(config, path.as_ptr().cast()) == 0 {
            warn!(dir = %fonts_dir.display(), "bundled font registration failed");
        } else {
            info!(dir = %fonts_dir.display(), "bundled fonts registered");
        }
    }
}

/// Locates `resources/fonts` — first next to the running executable
/// (installed layout), then relative to the crate (development layout).
fn bundled_fonts_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let candidate = parent.join("resources").join("fonts");
        if candidate.is_dir() {
            return candidate;
        }
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/fonts"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    /// Queries fontconfig for `family` after `install()` and returns the
    /// matched face's filename, if any.
    fn matched_file(family: &str) -> Option<String> {
        unsafe {
            let config = fc::FcConfigGetCurrent();
            let pattern = fc::FcNameParse(CString::new(family).ok()?.as_ptr().cast());
            let mut result = 0u32;
            let matched = fc::FcFontMatch(config, pattern, &mut result);
            if matched.is_null() || result != fc::FcResultMatch {
                return None;
            }
            let mut file: *mut fc::FcChar8 = std::ptr::null_mut();
            if fc::FcPatternGetString(matched, c"file".as_ptr(), 0, &mut file) != fc::FcResultMatch {
                return None;
            }
            let name = CStr::from_ptr(file.cast()).to_string_lossy().into_owned();
            Some(name)
        }
    }

    #[test]
    fn registers_bundled_fonts() {
        install();
        let matched = matched_file("Patrick Hand");
        assert!(matched.is_some(), "Patrick Hand did not resolve");
        assert!(matched.unwrap_or_default().contains("PatrickHand"), "resolved to an unexpected file");
    }
}
