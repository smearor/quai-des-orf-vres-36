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
pub mod label_spec;
pub mod luminance;
pub mod map_style;
pub mod neon;
pub mod palette;
pub mod path;
pub mod renderer;
pub mod station_spec;

pub use alpha::Alpha;
pub use belle_epoque::BelleEpoqueStyle;
pub use color::RgbColor;
pub use comic::ComicStyle;
pub use edge_spec::EdgeSpec;
pub use label_spec::LabelSpec;
pub use luminance::Luminance;
pub use map_style::MapStyle;
pub use neon::NeonStyle;
pub use palette::StylePalette;
pub use path::WATER_HALF_WIDTH;
pub use path::WaterBanks;
pub use path::band_path;
pub use path::edge_path;
pub use path::px_polyline_path;
pub use path::smooth_polyline;
pub use renderer::LabelOrientation;
pub use renderer::PxPos;
pub use renderer::StyleRenderer;
pub use station_spec::StationSpec;
