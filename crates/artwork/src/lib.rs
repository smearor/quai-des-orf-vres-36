//! Artwork variant generator for Quai des Orfèvres 36.
//!
//! Reads parametrized master SVGs and a `variants.toml` table, substitutes
//! the parametrized element attributes, inner markup and text contents per
//! variant, and writes one output SVG per variant. Intended for use both as
//! the `qdo36-artwork` binary and as a build dependency of other workspace
//! crates (for example inside `crates/app/build.rs`).

mod color;
mod engine;
mod error;
#[cfg(feature = "outline")]
mod fonts;
mod model;
#[cfg(feature = "outline")]
mod outline;

pub use crate::error::ArtworkError;
pub use crate::model::Asset;
pub use crate::model::InnerFill;
pub use crate::model::Spec;
pub use crate::model::StaticAsset;
pub use crate::model::Substitution;
pub use crate::model::TextFill;
pub use crate::model::Variant;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Generates all configured variants and returns the written file paths.
///
/// `config_path` points at a `variants.toml` table. Master SVG references
/// inside the table are resolved relative to the directory containing the
/// configuration file, so each `masters/{city}/{style}/` directory forms a
/// self-contained style pack of masters plus its own `variants.toml`.
/// `out_dir` is the directory the generated variants are written to
/// (created if absent).
///
/// # Errors
///
/// Returns [`ArtworkError`] when the configuration cannot be read or parsed,
/// when a master file is missing, when a parametrized element id or
/// attribute is absent from a master, or when an output file cannot be
/// written.
pub fn generate(config_path: &Path, out_dir: &Path) -> Result<Vec<PathBuf>, ArtworkError> {
    let config_source = fs::read_to_string(config_path).map_err(|source| ArtworkError::Io {
        path: config_path.to_owned(),
        source,
    })?;
    let spec: Spec = toml::from_str(&config_source)?;
    let city = spec.city.as_deref().unwrap_or("paris");
    let style = spec.style.as_deref().unwrap_or("belle-epoque");
    let masters_dir = config_path.parent().unwrap_or(Path::new("."));
    #[cfg(feature = "outline")]
    let fonts = if spec.outline.unwrap_or(true) {
        let fonts_dir = spec
            .fonts_dir
            .as_deref()
            .map(|dir| masters_dir.join(dir))
            .ok_or(ArtworkError::MissingFontsDir)?;
        Some(fonts::FontRegistry::load(&fonts_dir))
    } else {
        None
    };
    fs::create_dir_all(out_dir).map_err(|source| ArtworkError::Io {
        path: out_dir.to_owned(),
        source,
    })?;
    let mut written = Vec::new();
    for asset in &spec.assets {
        let master_path = masters_dir.join(&asset.master);
        let master_source = fs::read_to_string(&master_path).map_err(|source| ArtworkError::Io {
            path: master_path.clone(),
            source,
        })?;
        for variant in &asset.variants {
            let mut svg = master_source.clone();
            for substitution in &variant.substitutions {
                let value = substitution.resolve().to_string();
                svg = engine::set_attribute(&svg, &asset.master, &substitution.id, &substitution.attribute, &value)?;
            }
            for inner_fill in &variant.fills {
                svg = engine::set_inner(&svg, &asset.master, &inner_fill.id, &inner_fill.markup)?;
            }
            for text_fill in &variant.texts {
                svg = engine::set_text(&svg, &asset.master, &text_fill.id, &text_fill.content)?;
            }
            #[cfg(feature = "outline")]
            if let Some(fonts) = &fonts {
                svg = outline::outline_texts(&svg, fonts)?;
            }
            let file_name = asset.output.replace("{name}", &variant.name).replace("{city}", city).replace("{style}", style);
            let out_path = out_dir.join(&file_name);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|source| ArtworkError::Io {
                    path: parent.to_owned(),
                    source,
                })?;
            }
            fs::write(&out_path, &svg).map_err(|source| ArtworkError::Io {
                path: out_path.clone(),
                source,
            })?;
            written.push(out_path);
        }
    }
    #[cfg(feature = "outline")]
    if let Some(fonts) = &fonts {
        for static_asset in &spec.statics {
            let file_name = static_asset.path.replace("{city}", city).replace("{style}", style);
            let path = out_dir.join(&file_name);
            let source = fs::read_to_string(&path).map_err(|source| ArtworkError::Io { path: path.clone(), source })?;
            let outlined = outline::outline_texts(&source, fonts)?;
            fs::write(&path, &outlined).map_err(|source| ArtworkError::Io { path: path.clone(), source })?;
        }
    }
    Ok(written)
}
