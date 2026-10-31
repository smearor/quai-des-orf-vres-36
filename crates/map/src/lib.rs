//! Paris map domain for Quai des Orfèvres 36.
//!
//! The station graph the game is played on ([`GameMap`]), authored as a
//! reviewable TOML dataset ([`dataset`]) and validated at load time. All
//! positions and distances are in **millimeters** on the physical table
//! surface - pixel conversion happens in the app layer via the anchor
//! transform.

pub mod bounds;
pub mod dataset;
pub mod edge;
pub mod game_map;
pub mod map_error;
pub mod neighbor;
pub mod station;
pub mod station_id;
pub mod station_kind;
pub mod station_name;
pub mod transport;

pub use bounds::Bounds;
pub use edge::Edge;
pub use game_map::GameMap;
pub use map_error::MapError;
pub use neighbor::Neighbor;
pub use station::Station;
pub use station_id::StationId;
pub use station_kind::StationKind;
pub use station_name::StationName;
pub use tangible_objects_model::Millimeters;
pub use tangible_objects_model::Vec2;
pub use transport::Transport;
