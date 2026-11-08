//! Swappable map render styles (`ARTWORK.md` §2).
//!
//! `MapScene` owns the scene semantics (what exists: edges, stations,
//! backdrop, anchor); a [`StyleRenderer`] decides how it looks -
//! palette, stroke treatment, decorations, label font.

pub mod alpha;
pub mod belle_epoque;
pub mod block_spec;
pub mod color;
pub mod comic;
pub mod district_label;
pub mod edge_spec;
pub mod house_mode;
pub mod label_orientation;
pub mod label_spec;
pub mod luminance;
pub mod map_style;
pub mod neon;
pub mod outer_spec;
pub mod palette;
pub mod park_spec;
pub mod path;
pub mod px_polygon;
pub mod renderer;
pub mod station_spec;
pub mod to_px;

pub use alpha::Alpha;
pub use belle_epoque::BelleEpoqueStyle;
pub use block_spec::BlockSpec;
pub use color::RgbColor;
pub use comic::ComicStyle;
pub use district_label::DistrictLabel;
pub use edge_spec::EdgeSpec;
pub use house_mode::HouseMode;
pub use label_orientation::LabelOrientation;
pub use label_spec::LabelSpec;
pub use luminance::Luminance;
pub use map_style::MapStyle;
pub use neon::NeonStyle;
pub use outer_spec::OuterSpec;
pub use palette::StylePalette;
pub use park_spec::ParkSpot;
pub use path::BuildingPathExt;
pub use path::CaptionStyle;
pub use path::EdgePathExt;
pub use path::GroundExt;
pub use path::WATER_HALF_WIDTH;
pub use path::WaterPathExt;
pub use px_polygon::PxPolygon;
pub use qdo36_map::WaterBanks;
pub use renderer::PxPos;
pub use renderer::StyleRenderer;
pub use station_spec::StationSpec;
pub use to_px::ToPx;
