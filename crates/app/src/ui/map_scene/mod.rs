//! The map scene widget: renders the GameMap under `MapTransform`
//! (map-mm -> surface-mm -> widget-px) on every snapshot.

pub mod bridge_deck;
pub mod controls;
pub mod deck_side;
pub mod hop_share;
pub mod label_mode;
pub mod lane_constraint;
pub mod layer_name;
pub mod layer_toggle;
pub mod parallel_share;
pub mod pose_animator;
pub mod render_params;
pub mod render_stage;
pub mod scene_layers;
pub mod scene_state;
pub mod transport_layers;
pub mod two_finger;
pub mod widget;

use cairo::Context;
pub use controls::controls;
pub use deck_side::DeckSide;
pub use hop_share::HopShare;
pub use label_mode::LabelMode;
pub use lane_constraint::LaneConstraint;
pub use layer_name::LayerName;
pub use layer_toggle::LayerToggle;
pub use parallel_share::ParallelShare;
pub use pose_animator::PoseAnimator;
pub use render_params::RenderParams;
pub use render_stage::RenderStage;
pub use scene_layers::SceneLayers;
pub use scene_state::SceneState;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
pub use transport_layers::TransportLayers;
pub use two_finger::TwoFingerGesture;
pub use two_finger::TwoFingerStep;
pub use widget::MapScene;

use crate::ui::art::AnchorFrac;
use crate::ui::art::ArtFamily;
use crate::ui::art::ArtSpec;
use crate::ui::art::AssetStem;
use crate::ui::map_scene::bridge_deck::BridgeDeck;
use crate::ui::style::Alpha;
use crate::ui::style::EdgeSpec;
use crate::ui::style::LabelOrientation;
use crate::ui::style::LabelSpec;
use crate::ui::style::PxPos;
use crate::ui::style::StationSpec;
use crate::ui::style::WATER_HALF_WIDTH;
use crate::ui::style::smooth_polyline;
use qdo36_map::Edge;
use qdo36_map::GameMap;
use qdo36_map::LineName;
use qdo36_map::Segment;
use qdo36_map::StationId;
use qdo36_map::StationPair;
use qdo36_map::Transport;
use qdo36_tangibles::AnchorPose;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::ScaleFactor;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::Vec2;

/// Full scene render: paper, backdrop, edges, stations, anchor,
/// overlay - each element delegated to the active `StyleRenderer`.
/// `pose` is the interpolated anchor pose for this frame; `width`x
/// `height` is the widget size in px.
pub fn render(ctx: &Context, state: &SceneState, pose: AnchorPose, width: Pixels, height: Pixels) {
    let map = state.map.as_ref();
    let stage = RenderStage::new(state, pose, width, height);
    let to_px = |pos| stage.to_px(pos);
    let px_per_mm = stage.fit.scale;
    let renderer = state.style.renderer();

    // Paper.
    renderer.paper().apply(ctx);
    ctx.paint().ok();

    // Backdrop: water + landmarks (gameplay-inert dressing). The
    // Chaikin-smoothed courses feed both the river bands and the
    // Batobus routes, so band and boat line can never diverge.
    // Water vertices that sit on a `bridge` station stay pinned:
    // the river keeps flowing right under those markers.
    let backdrop = map.backdrop();
    let bridge_positions: Vec<Vec2<Millimeters>> = map.stations().filter(|station| station.is_bridge()).map(|station| station.pos).collect();
    let water_courses: Vec<Vec<Vec2<Millimeters>>> = backdrop
        .water
        .iter()
        .map(|polyline| {
            let pinned: Vec<usize> = polyline
                .points()
                .iter()
                .enumerate()
                .filter(|(_, point)| bridge_positions.iter().any(|pos| (pos.x - point.x).hypot(pos.y - point.y) < 1.5))
                .map(|(index, _)| index)
                .collect();
            smooth_polyline(polyline.points(), 2, &pinned)
        })
        .collect();
    if state.layers.water {
        for course in &water_courses {
            renderer.draw_water(ctx, course, px_per_mm, &to_px);
        }
    }
    // Bridge-station decks: one per `bridge` station, perpendicular
    // to the smoothed course, station marker mid-deck. Only while
    // the water layer is on - an invisible deck must not reroute
    // the roads either.
    let decks: HashMap<StationId, BridgeDeck> = if state.layers.water {
        BridgeDeck::decks(map, &water_courses, WATER_HALF_WIDTH)
    } else {
        HashMap::new()
    };
    // Smoothed water segments drive the bridge crossing tests.
    let water_segments: Vec<Segment<Millimeters>> = water_courses
        .iter()
        .flat_map(|course| course.windows(2).map(|pair| Segment::new(pair[0], pair[1])))
        .collect();
    let line_hidden = |edge: &Edge| edge.line.as_ref().is_some_and(|line| state.layers.hidden_lines.contains(line));
    if state.layers.landmarks {
        for landmark in &backdrop.landmarks {
            let pos = to_px(landmark.pos);
            let spec = ArtSpec {
                family: ArtFamily::Landmarks,
                stem: AssetStem::for_landmark(landmark.name.as_str()),
                pos,
                height: Millimeters::new(14.0 * state.params.landmark_scale.get()),
                anchor: AnchorFrac::PIN_TIP,
                alpha: Alpha::OPAQUE,
            };
            if !state.assets.draw(ctx, state.style, &spec, px_per_mm) {
                renderer.draw_landmark(ctx, pos, state.params.landmark_scale, px_per_mm);
            }
        }
    }

    // Batobus edges first: the boat course draws under the bridge
    // decks - boats pass under bridges, not over them.
    for edge in map.edges() {
        if edge.transport != Transport::Batobus || line_hidden(edge) || !state.layers.transports.visible(edge.transport) {
            continue;
        }
        let (Some(from), Some(to)) = (map.station(edge.from), map.station(edge.to)) else {
            continue;
        };
        // A bridgehead pier docks at its bank-side deck end - the
        // boat bends to the bank under its bridge instead of
        // cutting onto land. On-water piers keep mid-river.
        let dock = |id: StationId| {
            map.station(id).map(|station| {
                decks
                    .get(&id)
                    .filter(|deck| !deck.on_water)
                    .map_or(station.pos, |deck| deck.end_on_side(station.pos))
            })
        };
        let (mut route_mm, entry_arc) = if edge.via.is_empty() {
            match (dock(edge.from), dock(edge.to)) {
                (Some(a), Some(b)) => water_route(&water_courses, a, b),
                _ => (vec![from.pos, to.pos], 0.0),
            }
        } else {
            (
                [edge.from]
                    .into_iter()
                    .chain(edge.via.iter().copied())
                    .chain([edge.to])
                    .filter_map(dock)
                    .collect(),
                0.0,
            )
        };
        // The wave phase anchored to the course: the pier stub
        // shifts the local arc, so edges sharing a water stretch
        // trace the same wave instead of two crossing ones.
        let wave_phase = match (route_mm.first(), route_mm.get(1)) {
            (Some(a), Some(b)) => entry_arc - (b.x - a.x).hypot(b.y - a.y),
            _ => 0.0,
        };
        // Bridgehead docks bend into an arc: the water-course stub
        // inside the bend zone is trimmed, then two waypoints -
        // upstream of the deck center, part way toward the bank
        // end - turn the line in over open water before it slips
        // under the bridge.
        if route_mm.len() >= 2
            && let Some(deck) = decks.get(&edge.from).filter(|d| !d.on_water)
        {
            dock_arc(&mut route_mm, deck, true);
        }
        if route_mm.len() >= 2
            && let Some(deck) = decks.get(&edge.to).filter(|d| !d.on_water)
        {
            dock_arc(&mut route_mm, deck, false);
        }
        let route: Vec<PxPos> = route_mm.iter().map(|pos| to_px(*pos)).collect();
        renderer.draw_edge(
            ctx,
            &EdgeSpec {
                from: to_px(from.pos),
                to: to_px(to.pos),
                transport: edge.transport,
                line: edge.line.as_ref(),
                map,
                route,
                offsets: Vec::new(),
                smooth: true,
                dashed: false,
                wavy: true,
                wave_phase: Millimeters::new(wave_phase),
                highlight: state.layers.highlight.contains(&edge.transport) || state.layers.parallel_transports,
                px_per_mm,
            },
        );
    }

    // Bridge decks: where a visible taxi/bus edge crosses the
    // water, a pale plank under the road line (decorative). The
    // drawn route runs from -> via... -> to, so every hop segment
    // gets its own crossing test - `via` detours would otherwise
    // put the plank on the straight chord, off the drawn road.
    if state.layers.water {
        for edge in map.edges() {
            if !matches!(edge.transport, Transport::Taxi | Transport::Bus) || !state.layers.transports.visible(edge.transport) {
                continue;
            }
            let hop_ids: Vec<StationId> = [edge.from].into_iter().chain(edge.via.iter().copied()).chain([edge.to]).collect();
            let hops: Vec<Vec2<Millimeters>> = hop_ids.iter().filter_map(|id| map.station(*id)).map(|station| station.pos).collect();
            if hops.len() != hop_ids.len() {
                continue;
            }
            for (index, hop) in hops.windows(2).enumerate() {
                // A hop touching a bridge station rides on that
                // station's deck (on-water) or is bent onto it
                // (bridgehead) - no extra plank there.
                if decks.contains_key(&hop_ids[index]) || decks.contains_key(&hop_ids[index + 1]) {
                    continue;
                }
                let road = Segment::new(hop[0], hop[1]);
                // Strict crossing only - an edge merely ending on
                // the waterline is a pier access, not a bridge
                // (Q13 rule).
                let Some(cross) = water_segments
                    .iter()
                    .find_map(|water| if road.crosses(water) { road.intersection(water) } else { None })
                else {
                    continue;
                };
                let (dx, dy) = (road.to.x - road.from.x, road.to.y - road.from.y);
                if dx == 0.0 && dy == 0.0 {
                    continue;
                }
                let half = 6.0 / dx.hypot(dy);
                renderer.draw_bridge(
                    ctx,
                    to_px(Vec2::new(cross.x - dx * half, cross.y - dy * half)),
                    to_px(Vec2::new(cross.x + dx * half, cross.y + dy * half)),
                    px_per_mm,
                );
            }
        }
        // Bridge-station decks always draw - perpendicular to the
        // river, bank to bank, station marker mid-deck.
        for deck in decks.values() {
            let (from, to) = deck.ends();
            renderer.draw_bridge(ctx, to_px(from), to_px(to), px_per_mm);
        }
    }

    // Per-station served transports drive the service rings.
    let mut served: HashMap<StationId, Vec<Transport>> = HashMap::new();
    for edge in map.edges() {
        for id in [edge.from, edge.to] {
            let list = served.entry(id).or_default();
            if !list.contains(&edge.transport) {
                list.push(edge.transport);
            }
        }
    }

    // Edges: transport layer on + the line not hidden by name;
    // highlighted transports draw in their own colors, everything
    // else in neutral ink. Metro edges route through the street mesh
    // (or their declared `via`) as pass-through lines; hops shared
    // by distinct lines draw parallel offsets in each line's color.

    // Resolve every metro/underground edge's station path once
    // (metro: via > street mesh > chord, tunnels: via > chord) and
    // collect who shares each hop: metro by distinct line,
    // underground by edge - shared hops draw as parallel offsets
    // instead of piling identical strokes on top of each other.
    let mut routed_paths: Vec<(usize, Vec<StationId>)> = Vec::new();
    let mut sharing: HashMap<StationPair, Vec<HopShare>> = HashMap::new();
    for (edge_i, edge) in map.edges().iter().enumerate() {
        if !matches!(edge.transport, Transport::Metro | Transport::Underground) || line_hidden(edge) || !state.layers.transports.visible(edge.transport) {
            continue;
        }
        let path = if !edge.via.is_empty() {
            let mut path = vec![edge.from];
            path.extend_from_slice(&edge.via);
            path.push(edge.to);
            path
        } else if edge.transport == Transport::Underground || edge.direct {
            vec![edge.from, edge.to]
        } else {
            map.route_on_surface(edge.from, edge.to).unwrap_or_else(|| vec![edge.from, edge.to])
        };
        let share = match edge.transport {
            Transport::Metro => edge.line.clone().map(HopShare::Metro).unwrap_or(HopShare::Edge(edge_i)),
            _ => HopShare::Edge(edge_i),
        };
        for w in path.windows(2) {
            let bucket = sharing.entry(StationPair::new(w[0], w[1])).or_default();
            if !bucket.contains(&share) {
                bucket.push(share.clone());
            }
        }
        routed_paths.push((edge_i, path));
    }

    // Parallel transports: hops shared by several lanes draw side
    // by side, each lane in its own color (metro-line style). A
    // lane is a whole transport - taxi, bus; also metro while its
    // overlay is off - or, with the metro overlay on, a single
    // metro line in its line color.
    let metro_overlay = state.layers.highlight.contains(&Transport::Metro);
    let mut surface_sharing: HashMap<StationPair, BTreeSet<ParallelShare>> = HashMap::new();
    if state.layers.parallel_transports {
        for edge in map.edges() {
            if !matches!(edge.transport, Transport::Taxi | Transport::Bus) {
                continue;
            }
            let ids: Vec<StationId> = [edge.from].into_iter().chain(edge.via.iter().copied()).chain([edge.to]).collect();
            for hop in ids.windows(2) {
                let key = StationPair::new(hop[0], hop[1]);
                surface_sharing.entry(key).or_default().insert(ParallelShare::Transport(edge.transport));
            }
        }
        for (edge_i, path) in &routed_paths {
            let Some(edge) = map.edges().get(*edge_i).filter(|edge| edge.transport == Transport::Metro) else {
                continue;
            };
            let share = ParallelShare::for_metro(edge.line.as_ref(), metro_overlay);
            for hop in path.windows(2) {
                let key = StationPair::new(hop[0], hop[1]);
                surface_sharing.entry(key).or_default().insert(share.clone());
            }
        }
    }
    // Lane side per hop: the rank ordering on a hop may be
    // mirrored for all lanes at once (the lanes stay distinct);
    // the sign is chosen so lanes keep their side through via
    // vertices where possible. Without it, a hop traversed against
    // its canonical direction forces every lane to cross the
    // corridor at the vertex.
    let hop_normal = |key: StationPair| -> Option<Vec2<Millimeters>> {
        let (Some(a), Some(b)) = (map.station(key.a()), map.station(key.b())) else {
            return None;
        };
        let (dx, dy) = (b.pos.x - a.pos.x, b.pos.y - a.pos.y);
        let len = dx.hypot(dy);
        (len > 1e-9).then(|| Vec2::new(-dy / len, dx / len))
    };
    // One continuity vote per via vertex: side(v) should equal
    // side(u) * c, with c = sign(rank_u * rank_v * (n_u . n_v)).
    let mut constraints: Vec<LaneConstraint> = Vec::new();
    let mut collect = |hops: &[StationPair], ranks: &[f64]| {
        for i in 1..hops.len() {
            let (u, v) = (hops[i - 1], hops[i]);
            let (Some(n_u), Some(n_v)) = (hop_normal(u), hop_normal(v)) else { continue };
            let c = ranks[i - 1] * ranks[i] * (n_u.x * n_v.x + n_u.y * n_v.y);
            if c.abs() > 1e-9 {
                constraints.push(LaneConstraint {
                    from: u,
                    to: v,
                    relation: c.signum(),
                });
            }
        }
    };
    if state.layers.parallel_transports {
        for edge in map.edges() {
            if !matches!(edge.transport, Transport::Taxi | Transport::Bus) {
                continue;
            }
            let ids: Vec<StationId> = [edge.from].into_iter().chain(edge.via.iter().copied()).chain([edge.to]).collect();
            let hops: Vec<StationPair> = ids.windows(2).map(|w| StationPair::new(w[0], w[1])).collect();
            let share = ParallelShare::Transport(edge.transport);
            let ranks: Vec<f64> = hops
                .iter()
                .map(|key| {
                    surface_sharing
                        .get(key)
                        .map(|shares| centered_rank(shares.iter(), shares.len(), &share))
                        .unwrap_or(0.0)
                })
                .collect();
            collect(&hops, &ranks);
        }
        for (edge_i, path) in &routed_paths {
            let Some(edge) = map.edges().get(*edge_i).filter(|edge| edge.transport == Transport::Metro) else {
                continue;
            };
            let share = ParallelShare::for_metro(edge.line.as_ref(), metro_overlay);
            let hops: Vec<StationPair> = path.windows(2).map(|w| StationPair::new(w[0], w[1])).collect();
            let ranks: Vec<f64> = hops
                .iter()
                .map(|key| {
                    surface_sharing
                        .get(key)
                        .map(|shares| centered_rank(shares.iter(), shares.len(), &share))
                        .unwrap_or(0.0)
                })
                .collect();
            collect(&hops, &ranks);
        }
    } else {
        for (edge_i, path) in &routed_paths {
            let hops: Vec<StationPair> = path.windows(2).map(|w| StationPair::new(w[0], w[1])).collect();
            let Some(edge) = map.edges().get(*edge_i) else { continue };
            let share = match edge.transport {
                Transport::Metro => edge.line.clone().map(HopShare::Metro).unwrap_or(HopShare::Edge(*edge_i)),
                _ => HopShare::Edge(*edge_i),
            };
            let ranks: Vec<f64> = hops
                .iter()
                .map(|key| sharing.get(key).map(|shares| centered_rank(shares.iter(), shares.len(), &share)).unwrap_or(0.0))
                .collect();
            collect(&hops, &ranks);
        }
    }
    let mut hop_side: HashMap<StationPair, f64> = HashMap::new();
    for _ in 0..2 {
        for constraint in &constraints {
            let (from, to, relation) = (constraint.from, constraint.to, constraint.relation);
            match (hop_side.get(&from).copied(), hop_side.get(&to).copied()) {
                (Some(s_from), None) => {
                    hop_side.insert(to, s_from * relation);
                }
                (None, Some(s_to)) => {
                    hop_side.insert(from, s_to * relation);
                }
                (None, None) => {
                    hop_side.insert(from, 1.0);
                    hop_side.insert(to, relation);
                }
                _ => {}
            }
        }
    }
    // The hop's parallel offset for a lane: its rank among the
    // lanes serving the hop, centered - relative to the canonical
    // a < b direction, so opposite-direction traversals land on
    // different lanes instead of collapsing onto one - times the
    // hop's lane side (keeps lanes on their side at via vertices).
    let spacing = px_per_mm.scale(2.6);
    let surface_offset = |share: &ParallelShare, a: StationId, b: StationId| -> f64 {
        let key = StationPair::new(a, b);
        let dir = key.direction(a);
        let side = hop_side.get(&key).copied().unwrap_or(1.0);
        surface_sharing
            .get(&key)
            .filter(|shares| shares.len() > 1)
            .map(|shares| {
                let rank = shares.iter().position(|s| s == share).unwrap_or(0);
                (rank as f64 - (shares.len() as f64 - 1.0) / 2.0) * spacing * dir * side
            })
            .unwrap_or(0.0)
    };
    let metro_in_parallel = |edge: &Edge| edge.transport == Transport::Metro && state.layers.parallel_transports && !metro_overlay;

    for (edge_i, edge) in map.edges().iter().enumerate() {
        if edge.transport == Transport::Batobus || !state.layers.transports.visible(edge.transport) || line_hidden(edge) {
            continue;
        }
        let (Some(from), Some(to)) = (map.station(edge.from), map.station(edge.to)) else {
            continue;
        };
        // `via` waypoints draw any edge's route as a bent polyline
        // (metro/tunnel hops shared with others draw parallel
        // offsets). Batobus edges already drew under the bridges.
        let (route, offsets) = if matches!(edge.transport, Transport::Metro | Transport::Underground) {
            let path = routed_paths
                .iter()
                .find(|(e_i, _)| *e_i == edge_i)
                .map(|(_, path)| path.as_slice())
                .unwrap_or(&[]);
            let share = match edge.transport {
                Transport::Metro => edge.line.clone().map(HopShare::Metro).unwrap_or(HopShare::Edge(edge_i)),
                _ => HopShare::Edge(edge_i),
            };
            let offsets: Vec<Pixels> = if edge.transport == Transport::Metro && state.layers.parallel_transports {
                let share = ParallelShare::for_metro(edge.line.as_ref(), metro_overlay);
                path.windows(2).map(|w| Pixels::new(surface_offset(&share, w[0], w[1]))).collect()
            } else {
                path.windows(2)
                    .map(|w| {
                        let key = StationPair::new(w[0], w[1]);
                        let dir = key.direction(w[0]);
                        let side = hop_side.get(&key).copied().unwrap_or(1.0);
                        let (count, rank) = sharing
                            .get(&key)
                            .and_then(|bucket| bucket.iter().position(|s| *s == share).map(|rank| (bucket.len(), rank)))
                            .unwrap_or((1, 0));
                        Pixels::new((rank as f64 - (count as f64 - 1.0) / 2.0) * spacing * dir * side)
                    })
                    .collect()
            };
            let route: Vec<_> = path.iter().filter_map(|id| map.station(*id)).map(|s| to_px(s.pos)).collect();
            (route, offsets)
        } else {
            // Above-ground edges dock at bridge-station deck ends;
            // everything else follows its `via` stations (or the
            // straight chord when `via` is empty). Hops shared by
            // several transports draw parallel offsets in the
            // edge's own color (parallel-transports overlay).
            let ids: Vec<StationId> = [edge.from].into_iter().chain(edge.via.iter().copied()).chain([edge.to]).collect();
            let (route_mm, hop_tags) = road_route(&ids, map, &decks, &water_segments);
            let route: Vec<PxPos> = route_mm.iter().map(|pos| to_px(*pos)).collect();
            let offsets: Vec<Pixels> = if state.layers.parallel_transports {
                (0..route.len().saturating_sub(1))
                    .map(|segment| {
                        let hop = hop_tags[segment];
                        Pixels::new(surface_offset(&ParallelShare::Transport(edge.transport), ids[hop], ids[hop + 1]))
                    })
                    .collect()
            } else {
                Vec::new()
            };
            (route, offsets)
        };
        renderer.draw_edge(
            ctx,
            &EdgeSpec {
                from: to_px(from.pos),
                to: to_px(to.pos),
                transport: edge.transport,
                // A metro in parallel mode draws the generic metro
                // color, not per-line colors (those are the metro
                // overlay's job).
                line: if metro_in_parallel(edge) { None } else { edge.line.as_ref() },
                map,
                route,
                offsets,
                smooth: false,
                wavy: false,
                wave_phase: Millimeters::new(0.0),
                // A metro lane dashes while both overlays run, so
                // a line color matching a neighboring transport
                // lane stays readable.
                dashed: edge.transport == Transport::Metro && state.layers.parallel_transports && metro_overlay,
                highlight: state.layers.highlight.contains(&edge.transport)
                    || (state.layers.parallel_transports && matches!(edge.transport, Transport::Taxi | Transport::Bus))
                    || metro_in_parallel(edge),
                px_per_mm,
            },
        );
    }

    // Per-station served metro lines drive the overlay pie fill.
    let mut served_lines: HashMap<StationId, Vec<LineName>> = HashMap::new();
    for edge in map.edges() {
        if edge.transport == Transport::Metro
            && !line_hidden(edge)
            && let Some(line) = &edge.line
        {
            for id in [edge.from, edge.to] {
                let list = served_lines.entry(id).or_default();
                if !list.contains(line) {
                    list.push(line.clone());
                }
            }
        }
    }
    // Metro line badges on the routed paths: on the two edges
    // carrying the line's end stations plus every third segment.
    if metro_overlay {
        let mut lines: BTreeMap<LineName, Vec<Vec<StationId>>> = BTreeMap::new();
        for (edge_i, path) in &routed_paths {
            if let Some(line) = map.edges().get(*edge_i).and_then(|e| e.line.clone()) {
                lines.entry(line).or_default().push(path.clone());
            }
        }
        for (line, paths) in &lines {
            // Chain the line's paths into one station sequence by
            // shared endpoints.
            let mut chain = paths.first().cloned().unwrap_or_default();
            let mut rest: Vec<Vec<qdo36_map::StationId>> = paths.iter().skip(1).cloned().collect();
            while let Some(idx) = rest.iter().position(|p| p.first() == chain.last() || p.last() == chain.last()) {
                let mut piece = rest.swap_remove(idx);
                if piece.first() == chain.last() {
                    chain.extend(piece.into_iter().skip(1));
                } else {
                    piece.reverse();
                    chain.extend(piece.into_iter().skip(1));
                }
            }
            let segments = chain.len().saturating_sub(1);
            for i in 0..segments {
                if i != 0 && i != segments - 1 && i % 3 != 0 {
                    continue;
                }
                let (Some(a), Some(b)) = (map.station(chain[i]), map.station(chain[i + 1])) else {
                    continue;
                };
                // The badge rides on its own lane - the offset the
                // edge draws at - so stacked lines keep their
                // markers apart instead of printing on each other.
                let offset = if state.layers.parallel_transports {
                    surface_offset(&ParallelShare::for_metro(Some(line), metro_overlay), chain[i], chain[i + 1])
                } else {
                    let key = StationPair::new(chain[i], chain[i + 1]);
                    let dir = key.direction(chain[i]);
                    let side = hop_side.get(&key).copied().unwrap_or(1.0);
                    sharing
                        .get(&key)
                        .and_then(|bucket| {
                            bucket
                                .iter()
                                .position(|s| *s == HopShare::Metro(line.clone()))
                                .map(|rank| (rank as f64 - (bucket.len() as f64 - 1.0) / 2.0) * spacing * dir * side)
                        })
                        .unwrap_or(0.0)
                };
                let (a_px, b_px) = (to_px(a.pos), to_px(b.pos));
                let (dx, dy) = (b_px.x - a_px.x, b_px.y - a_px.y);
                let len = dx.hypot(dy).max(1e-6);
                let mid = PxPos::new((a_px.x + b_px.x) / 2.0 - dy / len * offset, (a_px.y + b_px.y) / 2.0 + dx / len * offset);
                let spec = ArtSpec {
                    family: ArtFamily::Ligne,
                    stem: AssetStem::for_line(line),
                    pos: mid,
                    height: Millimeters::new(6.0 * state.params.metro_scale.get()),
                    anchor: AnchorFrac::CENTER,
                    alpha: Alpha::OPAQUE,
                };
                state.assets.draw(ctx, state.style, &spec, px_per_mm);
            }
        }
    }

    // Stations + labels + numbers + scores.
    let importance = map.importance();
    // Radial labels face the nearest *screen* edge - the player's
    // seat side - not the map center (which pans off-viewport).
    let center = PxPos::new(width.as_f64() / 2.0, height.as_f64() / 2.0);
    let orientation = match state.layers.label_mode {
        LabelMode::Radial => LabelOrientation::Radial { center },
        LabelMode::Screen => LabelOrientation::Screen,
    };
    let radius = Pixels::new(px_per_mm.scale(3.0).max(0.4));
    for station in map.stations() {
        let pos = to_px(station.pos);
        let station_importance = importance.get(&station.id);
        if state.layers.stations {
            renderer.draw_station(
                ctx,
                &StationSpec {
                    pos,
                    served: served.get(&station.id).map(Vec::as_slice).unwrap_or_default(),
                    lines: served_lines.get(&station.id).map(Vec::as_slice).unwrap_or_default(),
                    map,
                    metro_overlay,
                    importance: station_importance.map(|i| i.class),
                    selected: state.selected == Some(station.id),
                    station_scale: state.params.station_scale,
                    ring_width: state.params.ring_width,
                    px_per_mm,
                },
            );
        }
        if state.layers.labels {
            renderer.draw_label(
                ctx,
                &LabelSpec {
                    text: station.name.as_str(),
                    pos,
                    radius,
                    orientation,
                    inside: false,
                    label_scale: state.params.label_scale,
                    px_per_mm,
                },
            );
        }
        if state.layers.numbers {
            renderer.draw_label(
                ctx,
                &LabelSpec {
                    text: &station.id.to_string(),
                    pos,
                    radius,
                    orientation,
                    inside: true,
                    label_scale: state.params.label_scale,
                    px_per_mm,
                },
            );
        }
        if state.layers.scores
            && let Some(info) = station_importance
        {
            // Pushed further out than the name label.
            renderer.draw_label(
                ctx,
                &LabelSpec {
                    text: &info.score.0.to_string(),
                    pos,
                    radius: Pixels::new(radius.as_f64() * 2.4),
                    orientation,
                    inside: false,
                    label_scale: state.params.label_scale,
                    px_per_mm,
                },
            );
        }
    }

    // Dataset-debug layer: ring every `bridge` station.
    if state.layers.bridges {
        for station in map.stations() {
            if station.is_bridge() {
                renderer.draw_landmark(ctx, to_px(station.pos), ScaleFactor::new(2.0), px_per_mm);
            }
        }
    }

    // Anchor marker: the Eiffel pin artwork where the style ships one,
    // else the primitive ring.
    let tracked = matches!(state.anchor, AnchorState::Tracked(_));
    let anchor_pos = to_px(state.config.anchor_map_pos);
    let anchor_spec = ArtSpec {
        family: ArtFamily::Landmarks,
        stem: AssetStem::new("eiffel-anchor"),
        pos: anchor_pos,
        height: Millimeters::new(16.0),
        anchor: AnchorFrac::PIN_TIP,
        alpha: if tracked { Alpha::OPAQUE } else { Alpha::new(0.4) },
    };
    if !state.assets.draw(ctx, state.style, &anchor_spec, px_per_mm) {
        renderer.draw_anchor(ctx, anchor_pos, tracked, px_per_mm);
    }

    // Compass rose: rotates with the map, so it always reads map-north.
    if state.layers.compass {
        let compass_pos = to_px(state.compass_pos);
        let compass_spec = ArtSpec {
            family: ArtFamily::Map,
            stem: AssetStem::new("compass"),
            pos: compass_pos,
            height: Millimeters::new(26.0),
            anchor: AnchorFrac::CENTER,
            alpha: Alpha::new(0.9),
        };
        state.assets.draw(ctx, state.style, &compass_spec, px_per_mm);
    }

    // "Tower missing" hint - only while virtual.
    if let (Some(hint), AnchorState::Virtual(_)) = (&state.anchor_hint, state.anchor) {
        renderer.draw_hint(ctx, hint, width, height);
    }
}

/// The pier-to-pier course a Batobus edge draws: both station
/// positions are projected onto the closest water course and the
/// route follows the waterline between the two projected points -
/// the boat hugs the river instead of cutting corners over land.
/// `water` carries the same smoothed courses the bands draw from.
/// Empty when the map ships no water.
/// The boat route plus the course arc length at the water-entry
/// point - the second value anchors the wave phase so edges
/// sharing a stretch trace identical waves.
fn water_route(water: &[Vec<Vec2<Millimeters>>], from: Vec2<Millimeters>, to: Vec2<Millimeters>) -> (Vec<Vec2<Millimeters>>, f64) {
    // The closest point on a polyline: (position along the line as
    // segment index + fraction, projected point, approach distance).
    let closest = |points: &[Vec2<Millimeters>], pos: Vec2<Millimeters>| {
        let mut best: Option<(f64, Vec2<Millimeters>, f64)> = None;
        for (index, window) in points.windows(2).enumerate() {
            let (vx, vy) = (window[1].x - window[0].x, window[1].y - window[0].y);
            let len_sq = vx * vx + vy * vy;
            if len_sq == 0.0 {
                continue;
            }
            let t = (((pos.x - window[0].x) * vx + (pos.y - window[0].y) * vy) / len_sq).clamp(0.0, 1.0);
            let projected = Vec2::new(window[0].x + t * vx, window[0].y + t * vy);
            let distance = (pos.x - projected.x).hypot(pos.y - projected.y);
            if best.is_none_or(|(_, _, d)| distance < d) {
                best = Some((index as f64 + t, projected, distance));
            }
        }
        best
    };
    let (mut best_score, mut best) = (f64::INFINITY, None);
    for points in water {
        let (Some(entry), Some(exit)) = (closest(points, from), closest(points, to)) else {
            continue;
        };
        let score = entry.2 + exit.2;
        if score < best_score {
            (best_score, best) = (score, Some((points, entry, exit)));
        }
    }
    let Some((points, entry, exit)) = best else { return (Vec::new(), 0.0) };
    // Follow the course from `entry` to `exit` - the inner course
    // vertices run in polyline order when `entry` is upstream, in
    // reverse otherwise.
    let (low, high) = if entry.0 <= exit.0 { (entry, exit) } else { (exit, entry) };
    let i0 = low.0.floor() as usize + 1;
    let i1 = high.0.floor() as usize;
    let inner: &[Vec2<Millimeters>] = if i0 <= i1 { &points[i0..=i1] } else { &[] };
    let mut route = vec![from, entry.1];
    if entry.0 <= exit.0 {
        route.extend_from_slice(inner);
    } else {
        route.extend(inner.iter().rev().copied());
    }
    route.extend_from_slice(&[exit.1, to]);
    // The entry point's arc length along the course anchors the
    // wave phase: edges sharing a stretch draw matching waves.
    let entry_arc = points
        .windows(2)
        .take(entry.0.floor() as usize)
        .map(|pair| (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y))
        .sum::<f64>()
        + {
            let i = entry.0.floor() as usize;
            let frac = entry.0.fract();
            let (a, b) = (points[i], points[(i + 1).min(points.len() - 1)]);
            (b.x - a.x).hypot(b.y - a.y) * frac
        };
    (route, entry_arc)
}

/// Bends a bridgehead dock into an arc on the water course: trims
/// the course points inside the bend zone (the raw route would
/// otherwise overshoot to the dock's projection and double back -
/// a hairpin), then splices two waypoints upstream of the deck
/// center along the river tangent, part way toward the bank-side
/// dock end. `at_start` docks the route's first point, `false`
/// its last.
fn dock_arc(route: &mut Vec<Vec2<Millimeters>>, deck: &BridgeDeck, at_start: bool) {
    const FAR: f64 = 5.5;
    let tangent: Vec2<Millimeters> = Vec2::new(-deck.axis.y, deck.axis.x);
    let t_of = |p: Vec2<Millimeters>| (p.x - deck.center.x) * tangent.x + (p.y - deck.center.y) * tangent.y;
    let Some(dock) = (if at_start { route.first() } else { route.last() }).copied() else {
        return;
    };
    let bend = |upstream_mm: f64, bank_frac: f64, side: f64| -> Vec2<Millimeters> {
        Vec2::new(
            deck.center.x + tangent.x * upstream_mm * side + (dock.x - deck.center.x) * bank_frac,
            deck.center.y + tangent.y * upstream_mm * side + (dock.y - deck.center.y) * bank_frac,
        )
    };
    let side_of = |route: &[Vec2<Millimeters>], index: usize| route.get(index).map(|w| if t_of(*w) >= 0.0 { 1.0 } else { -1.0 }).unwrap_or(1.0);
    if at_start {
        while route.len() > 2 && t_of(route[1]).abs() < FAR {
            route.remove(1);
        }
        let side = side_of(route, 1);
        route.splice(1..1, [bend(2.0, 0.7, side), bend(FAR, 0.3, side)]);
    } else {
        while route.len() > 2 && t_of(route[route.len() - 2]).abs() < FAR {
            route.remove(route.len() - 2);
        }
        let side = side_of(route, route.len() - 2);
        route.splice(route.len() - 1..route.len() - 1, [bend(FAR, 0.3, side), bend(2.0, 0.7, side)]);
    }
}

/// The drawn path of an above-ground edge (map mm): on-water
/// bridge-station endpoints snap to their deck end on the
/// neighbor's side - roads meet the bridge instead of drawing onto
/// the water. An interior on-water bridge station (`via`) enters
/// at the near deck end, passes the station mid-deck and leaves at
/// the far end. A hop that crosses water next to a bridgehead
/// station bends onto that station's deck, so roads always cross
/// on a bridge - never beside it.
/// Returns the route points plus, per point, the index of the
/// `ids` hop its outgoing segment belongs to - the parallel-
/// transports overlay shifts each segment by its hop's offset.
fn road_route(
    ids: &[StationId],
    map: &GameMap,
    decks: &HashMap<StationId, BridgeDeck>,
    water_segments: &[Segment<Millimeters>],
) -> (Vec<Vec2<Millimeters>>, Vec<usize>) {
    let station_pos = |index: usize| ids.get(index).and_then(|id| map.station(*id)).map(|station| station.pos);
    let mut route = Vec::with_capacity(ids.len() + 2);
    let mut hops = Vec::with_capacity(ids.len() + 2);
    for (i, id) in ids.iter().enumerate() {
        let Some(station) = map.station(*id) else { continue };
        match decks.get(id) {
            Some(deck) if deck.on_water && (i == 0 || i + 1 == ids.len()) => {
                let neighbor = station_pos(if i == 0 { 1 } else { i - 1 }).unwrap_or(station.pos);
                route.push(deck.end_on_side(neighbor));
                hops.push(i.saturating_sub(1));
            }
            Some(deck) if deck.on_water => {
                route.push(deck.end_on_side(station_pos(i - 1).unwrap_or(station.pos)));
                route.push(station.pos);
                route.push(deck.end_on_side(station_pos(i + 1).unwrap_or(station.pos)));
                hops.extend_from_slice(&[i - 1, i, i]);
            }
            _ => {
                route.push(station.pos);
                hops.push(i.min(ids.len().saturating_sub(2)));
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
        let Some(crossing) = water_segments
            .iter()
            .find_map(|water| if road.crosses(water) { road.intersection(water) } else { None })
        else {
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
        hops.extend_from_slice(&[i, i]);
    }
    (route, hops)
}

/// The share's centered rank offset among the hop's lanes.
fn centered_rank<'a, S: PartialEq + 'a>(shares: impl IntoIterator<Item = &'a S>, count: usize, want: &S) -> f64 {
    shares
        .into_iter()
        .position(|share| share == want)
        .map(|rank| rank as f64 - (count as f64 - 1.0) / 2.0)
        .unwrap_or(0.0)
}
