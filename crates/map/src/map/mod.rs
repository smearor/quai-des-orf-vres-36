//! The game map domain: the validated station graph, its routes and
//! transport modes, flavor text and loading errors.

pub mod dataset;
pub mod edge;
pub mod error;
pub mod game_map;
pub mod slogan;
pub mod story;
pub mod transport;

pub use edge::Edge;
pub use error::MapError;
pub use game_map::GameMap;
pub use slogan::MapSlogan;
pub use story::MapStory;
pub use transport::Transport;
