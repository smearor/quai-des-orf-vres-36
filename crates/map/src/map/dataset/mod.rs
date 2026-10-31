//! TOML deserialization shape of the map dataset (`data/paris.toml`).
//!
//! Pure serde types - [`crate::GameMap`] is produced by validating
//! a [`MapToml`] via `TryFrom`.

pub mod edge_toml;
pub mod map_meta;
pub mod map_toml;
pub mod station_toml;

pub use edge_toml::EdgeToml;
pub use map_meta::MapMeta;
pub use map_toml::MapToml;
pub use station_toml::StationToml;
