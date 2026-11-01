//! Error type of the artwork generator.

use std::path::PathBuf;

/// Error type of the artwork generator.
#[derive(Debug, thiserror::Error)]
pub enum ArtworkError {
    /// An input file could not be read or an output file could not be written.
    #[error("i/o error on {path}: {source}")]
    Io {
        /// The file path the failing operation targeted.
        path: PathBuf,
        /// The underlying i/o error.
        source: std::io::Error,
    },
    /// The variants configuration could not be parsed.
    #[error("invalid variants config: {0}")]
    Config(#[from] toml::de::Error),
    /// A parametrized element id was not found in the master SVG.
    #[error("element with id `{id}` not found in master `{master}`")]
    MissingElement {
        /// The master file the element was expected in.
        master: String,
        /// The parametrized element id that was not found.
        id: String,
    },
    /// The attribute to substitute is absent on the parametrized element.
    #[error("attribute `{attribute}` missing on element `{id}` in master `{master}`")]
    MissingAttribute {
        /// The master file the element lives in.
        master: String,
        /// The parametrized element id.
        id: String,
        /// The attribute that was not found on the element.
        attribute: String,
    },
    /// A colour value is not a `#rrggbb` hex literal.
    #[error("invalid colour value `{0}` (expected `#rrggbb`)")]
    InvalidColour(String),
    /// Text outlining is enabled but no `fonts_dir` is configured.
    #[error("`outline` enabled but no `fonts_dir` configured in variants.toml")]
    MissingFontsDir,
    /// No bundled font face matches the requested family/weight.
    #[error("no bundled font face matches family `{family}`")]
    MissingFontFamily {
        /// The font-family value that could not be resolved.
        family: String,
    },
}
