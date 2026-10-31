//! `qdo36-map` - the Paris (and future cities) game map domain.
//!
//! Loads a curated TOML dataset into a validated station graph
//! ([`GameMap`]): stations, undirected transport edges, coordinate
//! bounds in map millimeters, city rule defaults and the city's
//! anchor landmark spec. No GTK, no daemon - pure domain model.

pub mod anchor;
pub mod city;
pub mod map;
pub mod station;

pub use anchor::AnchorName;
pub use anchor::AnchorProfile;
pub use anchor::AnchorSpec;
pub use city::City;
pub use city::CityName;
pub use map::Edge;
pub use map::FugitiveAlias;
pub use map::FugitiveSpec;
pub use map::GameMap;
pub use map::LineColor;
pub use map::LineName;
pub use map::MapError;
pub use map::MapMeta;
pub use map::MapSlogan;
pub use map::MapStory;
pub use map::Transport;
pub use map::TransportName;
pub use station::Neighbor;
pub use station::Station;
pub use station::StationId;
pub use station::StationKind;
pub use station::StationName;
pub use tangible_objects_model::Millimeters;
pub use tangible_objects_model::Region;
pub use tangible_objects_model::Vec2;
