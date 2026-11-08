//! `serde_pair_option` - `[x, y]` pair serde for `Option<Vec2>`.
//!
//! Same flat pair shape as `tangible_objects_model::serde_pair`,
//! but tolerant of an absent position (`pos_mm` missing or `null`).

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use tangible_objects_model::Vec2;

/// Serializes `Some(p)` as `[p.x, p.y]`, `None` as nothing.
pub fn serialize<U, S: Serializer>(pos: &Option<Vec2<U>>, serializer: S) -> Result<S::Ok, S::Error> {
    match pos {
        Some(point) => [point.x, point.y].serialize(serializer),
        None => serializer.serialize_none(),
    }
}

/// Deserializes `[x, y]` into `Some(Vec2)`, absent into `None`.
pub fn deserialize<'de, U, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Vec2<U>>, D::Error> {
    <Option<[f64; 2]>>::deserialize(deserializer).map(|pair| pair.map(Vec2::from))
}
