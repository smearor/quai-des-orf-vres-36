//! `BridgeDeck` - the drawn bridge of a `bridge`-kind station.

use crate::GameMap;
use crate::Polyline;
use crate::Segment;
use crate::Station;
use crate::StationId;
use crate::StationKind;
use crate::StationPair;
use crate::map::DeckSide;
use crate::map::WaterBanks;
use crate::map::bridge::RoadRoute;
use crate::map::bridge::near_point::NearPoint;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use std::collections::HashMap;

/// How far a deck reaches across its water course, in river
/// half-widths: stations further out count as off-water, and a
/// deck axis looks no farther than this for its banks.
const REACH_HALF_WIDTHS: f64 = 4.0;

/// Extra span beyond the measured bank-to-bank distance - the
/// apron lands the deck ends on the banks (mm).
const SPAN_APRON: Millimeters = Millimeters::new(1.2);

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
    /// The deck at `near` on `course`: perpendicular to the bank
    /// segment, centered on `station` when it sits on the water,
    /// spanning bank to bank plus the apron. `None` on a
    /// degenerate course segment.
    pub(super) fn at(station: &Station, near: &NearPoint, course: &Polyline, river_half_width: Millimeters) -> Option<Self> {
        let course_segment = Segment::new(course[near.segment], course[near.segment + 1]);
        let direction = course_segment.to - course_segment.from;
        if direction.magnitude() <= Millimeters::new(1e-9) {
            return None;
        }
        let axis = direction.normalized().perpendicular();
        let on_water = near.distance <= river_half_width;
        let center = if on_water {
            station.pos
        } else {
            course_segment.from + direction * near.fraction
        };
        // The span: where the axis through the center meets each
        // bank; a short apron lands the ends on the banks.
        let banks = WaterBanks::new(course, river_half_width);
        let half_span = banks.crossing_span(center, axis, river_half_width * REACH_HALF_WIDTHS).max(river_half_width) + SPAN_APRON;
        Some(Self {
            center,
            axis,
            half_span,
            on_water,
        })
    }

    /// Which side of the deck `pos` lies on, measured along
    /// `axis`.
    pub fn side_of(&self, pos: Vec2<Millimeters>) -> DeckSide {
        let along = (pos - self.center).dot::<f64>(&self.axis);
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
        self.center + self.axis * (self.half_span.as_f64() * side)
    }

    /// Both deck ends.
    pub fn ends(&self) -> (Vec2<Millimeters>, Vec2<Millimeters>) {
        (self.center - self.axis * self.half_span.as_f64(), self.center + self.axis * self.half_span.as_f64())
    }

    /// Docks a batobus route's first point onto this deck: drops
    /// the course points inside the bend zone (the raw route would
    /// otherwise overshoot to the dock's projection and double
    /// back - a hairpin), then splices two waypoints upstream of
    /// the deck center along the river tangent, part way toward
    /// the bank-side dock end.
    pub fn dock_start(&self, route: &mut Polyline) {
        self.dock(route, true);
    }

    /// Docks a batobus route's last point onto this deck - the
    /// end counterpart of [`Self::dock_start`].
    pub fn dock_end(&self, route: &mut Polyline) {
        self.dock(route, false);
    }

    /// The docking bend: `at_start` docks the route's first
    /// point, `false` its last.
    fn dock(&self, route: &mut Polyline, at_start: bool) {
        const FAR: Millimeters = Millimeters::new(5.5);
        let deck = self;
        let tangent: Vec2<Millimeters> = deck.axis.perpendicular();
        let t_of = |p: Vec2<Millimeters>| (p - deck.center).dot::<Millimeters>(&tangent);
        let Some(dock) = (if at_start { route.first() } else { route.last() }).copied() else {
            return;
        };
        let bend = |upstream: Millimeters, bank_frac: f64, side: f64| -> Vec2<Millimeters> {
            deck.center + tangent * (upstream.as_f64() * side) + (dock - deck.center) * bank_frac
        };
        let side_of =
            |route: &[Vec2<Millimeters>], index: usize| route.get(index).map(|w| if t_of(*w) >= Millimeters::ZERO { 1.0 } else { -1.0 }).unwrap_or(1.0);
        if at_start {
            while route.len() > 2 && t_of(route[1]).abs() < FAR {
                route.0.remove(1);
            }
            let side = side_of(route, 1);
            route.0.splice(1..1, [bend(Millimeters::new(2.0), 0.7, side), bend(FAR, 0.3, side)]);
        } else {
            while route.len() > 2 && t_of(route[route.len() - 2]).abs() < FAR {
                route.0.remove(route.len() - 2);
            }
            let side = side_of(route, route.len() - 2);
            route
                .0
                .splice(route.len() - 1..route.len() - 1, [bend(FAR, 0.3, side), bend(Millimeters::new(2.0), 0.7, side)]);
        }
    }
}

impl GameMap {
    /// One `BridgeDeck` per `bridge`-kind station near water,
    /// computed against the smoothed courses the bands draw from.
    /// A station inside the band is `on_water` (it is the bridge);
    /// one within `4 * river_half_width` is a bridgehead (deck on
    /// the water at its projection); anything further is land - its
    /// `bridge` kind serves the dataset crossing rules only and
    /// gets no deck.
    pub fn bridge_decks(&self, river_half_width: Millimeters) -> HashMap<StationId, BridgeDeck> {
        let water = self.smoothed_water();
        let mut decks = HashMap::new();
        for station in self.stations_of_kind(StationKind::Bridge) {
            // The closest point on any course.
            let mut best: Option<NearPoint> = None;
            for (course_i, course) in water.iter().enumerate() {
                let Some((seg_i, nearest)) = course.nearest_segment(&station.pos) else {
                    continue;
                };
                let distance = station.pos.distance(&nearest.point);
                if best.as_ref().is_none_or(|best| distance < best.distance) {
                    best = Some(NearPoint {
                        distance,
                        course: course_i,
                        segment: seg_i,
                        fraction: nearest.fraction,
                    });
                }
            }
            let Some(near) = best else { continue };
            if near.distance > river_half_width * REACH_HALF_WIDTHS {
                continue;
            }
            if let Some(deck) = BridgeDeck::at(station, &near, &water[near.course], river_half_width) {
                decks.insert(station.id, deck);
            }
        }
        decks
    }

    /// The drawn path of an above-ground edge (map mm): on-water
    /// bridge-station endpoints snap to their deck end on the
    /// neighbor's side - roads meet the bridge instead of drawing
    /// onto the water. An interior on-water bridge station (`via`)
    /// enters at the near deck end, passes the station mid-deck and
    /// leaves at the far end. A hop that crosses water next to a
    /// bridgehead station bends onto that station's deck, so roads
    /// always cross on a bridge - never beside it.
    /// Returns the route points plus, per point, the index of the
    /// `ids` hop its outgoing segment belongs to - the parallel-
    /// transports overlay shifts each segment by its hop's offset.
    pub fn road_route(&self, ids: &[StationId], decks: &HashMap<StationId, BridgeDeck>, water_segments: &[Segment<Millimeters>]) -> RoadRoute {
        let station_pos = |index: usize| ids.get(index).and_then(|id| self.station(*id)).map(|station| station.pos);
        let mut route = Vec::with_capacity(ids.len() + 2);
        let mut hops = Vec::with_capacity(ids.len() + 2);
        let hop_pair = |hop: usize| StationPair::new(ids[hop], ids[hop + 1]);
        for (i, id) in ids.iter().enumerate() {
            let Some(station) = self.station(*id) else { continue };
            match decks.get(id) {
                Some(deck) if deck.on_water && (i == 0 || i + 1 == ids.len()) => {
                    let neighbor = station_pos(if i == 0 { 1 } else { i - 1 }).unwrap_or(station.pos);
                    route.push(deck.end_on_side(neighbor));
                    hops.push(hop_pair(i.saturating_sub(1)));
                }
                Some(deck) if deck.on_water => {
                    route.push(deck.end_on_side(station_pos(i - 1).unwrap_or(station.pos)));
                    route.push(station.pos);
                    route.push(deck.end_on_side(station_pos(i + 1).unwrap_or(station.pos)));
                    hops.extend([hop_pair(i - 1), hop_pair(i), hop_pair(i)]);
                }
                _ => {
                    route.push(station.pos);
                    hops.push(hop_pair(i.min(ids.len().saturating_sub(2))));
                }
            }
            // Bend the next hop onto a bridge: when the drawn segment
            // crosses water and a hop endpoint owns a bridgehead deck,
            // the road enters the near deck end and leaves the far one.
            let (Some(&from), Some(to_raw)) = (route.last(), station_pos(i + 1)) else {
                continue;
            };
            // An on-water endpoint snaps to its deck end - the drawn
            // hop ends on the bank and crosses nothing.
            let to = match ids.get(i + 1).and_then(|next| decks.get(next)).filter(|deck| deck.on_water) {
                Some(deck) => deck.end_on_side(from),
                None => to_raw,
            };
            let road = Segment::new(from, to);
            let Some(crossing) = water_segments.iter().find_map(|water| road.crossing_point(water)) else {
                continue;
            };
            let Some(deck) = [decks.get(id), ids.get(i + 1).and_then(|next| decks.get(next))]
                .into_iter()
                .flatten()
                .filter(|deck| !deck.on_water)
                .min_by(|a, b| {
                    let da = (a.center.x - crossing.x).hypot(a.center.y - crossing.y);
                    let db = (b.center.x - crossing.x).hypot(b.center.y - crossing.y);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
            else {
                continue;
            };
            route.push(deck.end_on_side(from));
            route.push(deck.end_on_side(to));
            hops.extend([hop_pair(i), hop_pair(i)]);
        }
        RoadRoute {
            polyline: Polyline::new(route),
            hops,
        }
    }
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
