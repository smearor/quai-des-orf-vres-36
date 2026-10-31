//! The game map domain: the validated station graph, its routes and
//! transport modes, flavor text, city rule defaults and loading
//! errors.

pub mod edge;
pub mod error;
pub mod fugitive;
pub mod game_map;
pub mod meta;
pub mod slogan;
pub mod story;
pub mod toml;
pub mod transport;

pub use edge::Edge;
pub use error::MapError;
pub use fugitive::FugitiveAlias;
pub use game_map::GameMap;
pub use meta::MapMeta;
pub use slogan::MapSlogan;
pub use story::MapStory;
pub use toml::MapToml;
pub use transport::Transport;
