//! The game map domain: the validated station graph, its routes and
//! transport modes, flavor text, city rule defaults and loading
//! errors.

pub mod backdrop;
pub mod cost;
pub mod edge;
pub mod edge_count;
pub mod error;
pub mod fugitive;
pub mod game_map;
pub mod line_color;
pub mod line_name;
pub mod meta;
pub mod segment;
pub mod slogan;
pub mod story;
pub mod toml;
pub mod transport;
pub mod transport_name;

pub use backdrop::Backdrop;
pub use backdrop::Landmark;
pub use backdrop::LandmarkKind;
pub use backdrop::LandmarkName;
pub use backdrop::Polyline;
pub use edge::Edge;
pub use edge_count::EdgeCount;
pub use error::MapError;
pub use fugitive::FugitiveAlias;
pub use fugitive::FugitiveSpec;
pub use game_map::GameMap;
pub use line_color::LineColor;
pub use line_name::LineName;
pub use meta::MapMeta;
pub use segment::Segment;
pub use slogan::MapSlogan;
pub use story::MapStory;
pub use toml::MapToml;
pub use transport::Transport;
pub use transport::UnknownTransport;
pub use transport_name::TransportName;
