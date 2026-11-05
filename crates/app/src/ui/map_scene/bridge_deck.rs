//! `BridgeDeck` - the drawn bridge of a `bridge`-kind station.

use crate::ui::map_scene::DeckSide;
use crate::ui::style::WaterBanks;
use qdo36_map::GameMap;
use qdo36_map::Segment;
use qdo36_map::StationId;
use std::collections::HashMap;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// The drawn bridge of a `bridge`-kind station: a deck
/// perpendicular to the river, spanning bank to bank. Two cases:
/// `on_water` stations sit on the water - the deck centers on the
/// station marker and above-ground (taxi/bus) edges dock at its
/// ends. Bridgehead stations stand on a bank - the deck centers on
/// the water below them and their edges stay ordinary roads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BridgeDeck {
    /// Deck center (map mm): the station position when `on_water`,
    /// its projection on the river course for bridgeheads.
    pub center: Vec2<Millimeters>,
    /// Unit deck axis: perpendicular to the river course.
    pub axis: Vec2<Millimeters>,
    /// Half the deck length.
    pub half_span: Millimeters,
    /// The station itself is the bridge (marker mid-river) - only
    /// those decks take edge attachments.
    pub on_water: bool,
}

impl BridgeDeck {
    /// Which side of the deck `pos` lies on, measured along
    /// `axis`.
    pub fn side_of(&self, pos: Vec2<Millimeters>) -> DeckSide {
        let along = (pos.x - self.center.x) * self.axis.x + (pos.y - self.center.y) * self.axis.y;
        if along > 0.0 {
            DeckSide::Along
        } else if along < 0.0 {
            DeckSide::Against
        } else {
            DeckSide::Center
        }
    }

    /// The deck end on `pos`'s side of the river - where an
    /// above-ground edge attaches. A centered position lands on
    /// the `along` end.
    pub fn end_on_side(&self, pos: Vec2<Millimeters>) -> Vec2<Millimeters> {
        let side = if self.side_of(pos).factor() < 0.0 { -1.0 } else { 1.0 };
        Vec2::new(
            self.center.x + self.axis.x * self.half_span.as_f64() * side,
            self.center.y + self.axis.y * self.half_span.as_f64() * side,
        )
    }

    /// Both deck ends.
    pub fn ends(&self) -> (Vec2<Millimeters>, Vec2<Millimeters>) {
        (
            Vec2::new(self.center.x - self.axis.x * self.half_span.as_f64(), self.center.y - self.axis.y * self.half_span.as_f64()),
            Vec2::new(self.center.x + self.axis.x * self.half_span.as_f64(), self.center.y + self.axis.y * self.half_span.as_f64()),
        )
    }
    /// One `BridgeDeck` per `bridge`-kind station near water,
    /// computed against the smoothed courses the bands draw from.
    /// A station inside the band is `on_water` (it is the bridge);
    /// one within `4 * river_half_width` is a bridgehead (deck on
    /// the water at its projection); anything further is land - its
    /// `bridge` kind serves the dataset crossing rules only and
    /// gets no deck.
    pub fn decks(map: &GameMap, water: &[Vec<Vec2<Millimeters>>], river_half_width: Millimeters) -> HashMap<StationId, Self> {
        let mut decks = HashMap::new();
        for station in map.stations() {
            if !station.is_bridge() {
                continue;
            }
            // The closest point on any course: (distance, course, segment, t).
            let mut best: Option<(f64, usize, usize, f64)> = None;
            for (course_i, course) in water.iter().enumerate() {
                for (seg_i, window) in course.windows(2).enumerate() {
                    let (vx, vy) = (window[1].x - window[0].x, window[1].y - window[0].y);
                    let len_sq = vx * vx + vy * vy;
                    if len_sq == 0.0 {
                        continue;
                    }
                    let t = (((station.pos.x - window[0].x) * vx + (station.pos.y - window[0].y) * vy) / len_sq).clamp(0.0, 1.0);
                    let distance = (station.pos.x - window[0].x - t * vx).hypot(station.pos.y - window[0].y - t * vy);
                    if best.is_none_or(|(d, _, _, _)| distance < d) {
                        best = Some((distance, course_i, seg_i, t));
                    }
                }
            }
            let Some((distance, course_i, seg_i, t)) = best else { continue };
            if distance > river_half_width.as_f64() * 4.0 {
                continue;
            }
            let course = &water[course_i];
            let (from, to) = (course[seg_i], course[seg_i + 1]);
            let (tx, ty) = (to.x - from.x, to.y - from.y);
            let length = tx.hypot(ty);
            if length == 0.0 {
                continue;
            }
            let axis = Vec2::new(-ty / length, tx / length);
            let on_water = distance <= river_half_width.as_f64();
            let center = if on_water { station.pos } else { Vec2::new(from.x + t * tx, from.y + t * ty) };
            // Deck span: where the axis through the center meets each
            // bank; a short apron lands the ends on the banks.
            let banks = WaterBanks::new(course, river_half_width);
            let span = [banks.left, banks.right]
                .iter()
                .filter_map(|bank| cross_param(center, axis, river_half_width.as_f64() * 4.0, bank))
                .fold(river_half_width.as_f64(), |acc, t| acc.max(t.abs()))
                + 1.2;
            decks.insert(
                station.id,
                BridgeDeck {
                    center,
                    axis,
                    half_span: Millimeters::new(span),
                    on_water,
                },
            );
        }
        decks
    }
}

/// The axis parameter `t` where the line `center + axis * t` meets
/// `bank` - `None` when they do not intersect within `reach`.
fn cross_param(center: Vec2<Millimeters>, axis: Vec2<Millimeters>, reach: f64, bank: &[Vec2<Millimeters>]) -> Option<f64> {
    let line = Segment::new(
        Vec2::new(center.x - axis.x * reach, center.y - axis.y * reach),
        Vec2::new(center.x + axis.x * reach, center.y + axis.y * reach),
    );
    bank.windows(2).find_map(|window| {
        line.intersection(&Segment::new(window[0], window[1]))
            .map(|point| (point.x - center.x) * axis.x + (point.y - center.y) * axis.y)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck() -> BridgeDeck {
        // River flows +x; deck crosses it vertically.
        BridgeDeck {
            center: Vec2::new(50.0, 100.0),
            axis: Vec2::new(0.0, 1.0),
            half_span: Millimeters::new(6.0),
            on_water: true,
        }
    }

    #[test]
    fn ends_are_symmetric_around_the_station() {
        let (a, b) = deck().ends();
        assert!((a.x - 50.0).abs() < 1e-9 && (a.y - 94.0).abs() < 1e-9);
        assert!((b.x - 50.0).abs() < 1e-9 && (b.y - 106.0).abs() < 1e-9);
    }

    #[test]
    fn end_on_side_picks_the_neighbors_bank_side() {
        let deck = deck();
        let north = deck.end_on_side(Vec2::new(50.0, 80.0));
        let south = deck.end_on_side(Vec2::new(50.0, 120.0));
        assert!((north.y - 94.0).abs() < 1e-9);
        assert!((south.y - 106.0).abs() < 1e-9);
    }
}
