//! `District` - a named city quarter anchor from the map dataset.
//!
//! Every inner block (the free areas the houses are drawn into)
//! gets a quarter name so players can communicate positions later
//! ("Der Corbeau war in Saint-Germain"). A district with `pos`
//! pins its name to the face containing that point - keeping the
//! quarters geographically close to reality. Districts without a
//! position fill the still-unnamed inner blocks in dataset order.

pub mod districts;
pub mod name;
pub use districts::Districts;
pub use name::DistrictName;

use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// A named city quarter, optionally pinned to a map position.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct District {
    /// Quarter name (proper noun, not localized).
    pub name: DistrictName,
    /// Position in map millimeters the quarter sits at; absent for
    /// names the layout assigns freely.
    #[serde(rename = "pos_mm", default, with = "crate::map::serde_pair_option")]
    pub pos: Option<Vec2<Millimeters>>,
}

impl District {
    /// Whether the dataset pins the quarter to a map position -
    /// `false` for names the layout assigns freely.
    pub fn is_positioned(&self) -> bool {
        self.pos.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_district_without_pos_still_parses() {
        let district: District = toml::from_str(r#"name = "Sentier""#).unwrap();
        assert_eq!(district.pos, None);
    }

    #[test]
    fn a_district_parses_its_pos() {
        let district: District = toml::from_str(
            r#"name = "Montmartre"
pos_mm = [240.0, 85.0]"#,
        )
        .unwrap();
        assert_eq!(district.pos.map(|p| (p.x, p.y)), Some((240.0, 85.0)));
    }
}
