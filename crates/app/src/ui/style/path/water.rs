//! Water geometry: river bands, banks and course smoothing in map mm.

/// Half the river band width - shared by the water bands and the
/// bridge-station decks, so every part of the scene agrees how
/// wide the river is.
pub use qdo36_map::WATER_HALF_WIDTH;

use cairo::Context;

use crate::ui::style::PxPos;

/// Cairo water paths: `ctx.band_path(..)` builds the filled river
/// body between two bank lines.
pub trait WaterPathExt {
    /// The closed band between two bank lines - the filled river
    /// body.
    fn band_path(&self, left_bank: &[PxPos], right_bank: &[PxPos]);
}

impl WaterPathExt for Context {
    fn band_path(&self, left_bank: &[PxPos], right_bank: &[PxPos]) {
        self.new_path();
        let Some(first) = left_bank.first() else { return };
        self.move_to(first.x, first.y);
        for point in &left_bank[1..] {
            self.line_to(point.x, point.y);
        }
        for point in right_bank.iter().rev() {
            self.line_to(point.x, point.y);
        }
        self.close_path();
    }
}
