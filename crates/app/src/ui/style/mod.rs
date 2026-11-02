//! Swappable map render styles (`ARTWORK.md` §2).
//!
//! `MapScene` owns the scene semantics (what exists: edges, stations,
//! backdrop, anchor); a [`StyleRenderer`] decides how it looks -
//! palette, stroke treatment, decorations, label font.

pub mod alpha;
pub mod belle_epoque;
pub mod color;
pub mod comic;
pub mod edge_spec;
pub mod map_style;
pub mod neon;
pub mod palette;
pub mod path;
pub mod renderer;

pub use alpha::Alpha;
pub use belle_epoque::BelleEpoqueStyle;
pub use color::RgbColor;
pub use comic::ComicStyle;
pub use edge_spec::EdgeSpec;
pub use map_style::MapStyle;
pub use neon::NeonStyle;
pub use palette::StylePalette;
pub use path::polyline_path;
pub use renderer::PxPos;
pub use renderer::StyleRenderer;
