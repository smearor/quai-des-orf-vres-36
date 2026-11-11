//! The map scene widget: renders the GameMap under `MapTransform`
//! (map-mm -> surface-mm -> widget-px) on every snapshot.

pub mod blocks;
pub mod controls;
pub mod label_mode;
pub mod lane;
pub mod layer;
pub mod pose_animator;
pub mod render_stage;
pub mod route;
pub mod scene;
pub mod share;
pub mod two_finger;
pub mod widget;

use cairo::Context;
pub use controls::controls;
pub use label_mode::LabelMode;
pub use lane::LaneConstraint;
pub use lane::LaneSides;
pub use lane::Sharing;
pub use layer::LayerName;
pub use layer::LayerToggle;
pub use pose_animator::PoseAnimator;
pub use render_stage::RenderStage;
pub use route::RoutePlan;
pub use route::WaterRoute;
pub use scene::RenderParams;
pub use scene::SceneLayers;
pub use scene::SceneState;
pub use scene::TransportLayers;
pub use share::HopShare;
pub use share::ParallelShare;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::f64::consts::FRAC_PI_2;
use std::f64::consts::TAU;
use std::rc::Rc;
pub use two_finger::TwoFingerGesture;
pub use two_finger::TwoFingerStep;
pub use widget::MapScene;

use crate::ui::art::AnchorFrac;
use crate::ui::art::ArtFamily;
use crate::ui::art::ArtSpec;
use crate::ui::art::AssetStem;
use crate::ui::style::EdgePathExt;
use crate::ui::style::EdgeSpec;
use crate::ui::style::LabelSpec;
use crate::ui::style::OuterSpec;
use crate::ui::style::PxPos;
use crate::ui::style::StationSpec;
use crate::ui::style::ToPx;
use crate::ui::style::WATER_HALF_WIDTH;
use qdo36_map::Edge;
use qdo36_map::EdgePath;
use qdo36_map::LineName;
use qdo36_map::Polygon;
use qdo36_map::Polyline;
use qdo36_map::Segment;
use qdo36_map::StationId;
use qdo36_map::StationKind;
use qdo36_map::StationPair;
use qdo36_map::Transport;
use qdo36_tangibles::AnchorPose;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::ScaleFactor;
use smearor_commons_color::Alpha;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::Vec2;

/// The station disc's nominal radius - labels and the
/// landmark-obstacle rings size off it (obstacles pad ~10 %).
const STATION_RADIUS_MM: Millimeters = Millimeters::new(3.0);
/// Half the drawn bridge glyph's span along its road (mm).
const BRIDGE_HALF_SPAN: Millimeters = Millimeters::new(6.0);
/// Base height of a metro line badge before `metro_scale` (mm).
const LIGNE_BADGE_HEIGHT: Millimeters = Millimeters::new(6.0);
/// Height of the anchor (Eiffel tower) glyph (mm).
const ANCHOR_HEIGHT: Millimeters = Millimeters::new(16.0);
/// Height of the compass rose glyph (mm).
const COMPASS_HEIGHT: Millimeters = Millimeters::new(26.0);

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
    // Chaikin-smoothed courses feed the river bands, the Batobus
    // routes and the block cuts, so band, boat line and block
    // outline can never diverge. Water vertices that sit on a
    // `bridge` station stay pinned: the river keeps flowing right
    // under those markers.
    let backdrop = map.backdrop();
    let water_courses = state.water_courses();

    // Countryside texture + city ground: the style's infinite
    // outer texture covers everything the ground polygons don't -
    // drawn under the Seine so the river washes its textured
    // banks too, and cached per view mode like the blocks.
    let blocks_geometry = if state.layers.blocks {
        let metro_overlay = state.layers.highlight.contains(&Transport::Metro);
        let parallel = state.layers.parallel_transports;
        let key = blocks::blocks_key(parallel, metro_overlay);
        let cached = state.block_shapes.borrow();
        let geometry = match cached.as_ref().filter(|(k, _)| *k == key) {
            Some((_, geometry)) => geometry.clone(),
            None => {
                drop(cached);
                let computed = Rc::new(blocks::BlocksGeometry::compute(state.block_layout(), parallel, metro_overlay));
                *state.block_shapes.borrow_mut() = Some((key, computed.clone()));
                computed
            }
        };
        renderer.draw_outer(
            ctx,
            &OuterSpec {
                ground: &geometry.ground,
                pin: Vec2::ZERO,
                width,
                height,
                px_per_mm,
            },
            ToPx(&to_px),
        );
        Some(geometry)
    } else {
        None
    };

    if state.layers.water {
        for course in water_courses {
            renderer.draw_water(ctx, course, px_per_mm, ToPx(&to_px));
        }
    }

    // City blocks: the free areas the streets, river and border
    // cut the map into. Drawn under everything else - lines,
    // labels and landmarks stay on top. The insets widen with the
    // street corridors in the parallel/metro overlays.
    if let Some(geometry) = &blocks_geometry {
        geometry.draw(ctx, renderer.as_ref(), &state.layers, PxPos::from_px(width / 2.0, height / 2.0), px_per_mm, ToPx(&to_px));
    }
    // Bridge-station decks for docking and drawing.
    let decks = state.decks();
    // Smoothed water segments drive the bridge crossing tests.
    let water_segments = state.water_segments();
    let line_hidden = |edge: &Edge| edge.line.as_ref().is_some_and(|line| state.layers.hidden_lines.contains(line));
    // Radial orientation faces away from the *screen* center - the
    // player's seat side - like the radial labels below.
    let center = PxPos::from_px(width / 2.0, height / 2.0);
    if state.layers.landmarks {
        // Obstacle geometry the pins dodge: every station disc and
        // each edge's station-chain chords (px space).
        let station_discs = station_discs(state, &stage);
        let edge_segments: Vec<(PxPos, PxPos)> = map
            .edges()
            .iter()
            .flat_map(|edge| {
                let ids: Vec<StationId> = edge.station_chain().collect();
                ids.windows(2)
                    .filter_map(|w| {
                        let (Some(a), Some(b)) = (map.station(w[0]), map.station(w[1])) else {
                            return None;
                        };
                        Some((to_px(a.pos), to_px(b.pos)))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        for landmark in &backdrop.landmarks {
            let pos = to_px(landmark.pos);
            let height_mm = 14.0 * state.params.landmark_scale.get();
            let preferred = state.layers.landmark_mode.preferred(pos, center);
            let angle = landmark_rotation(
                pos,
                preferred,
                px_per_mm.scale(Millimeters::new(height_mm * 0.5)).as_f64(),
                px_per_mm.scale(Millimeters::new(height_mm * 0.4)).as_f64(),
                &station_discs,
                &edge_segments,
            );
            let spec = ArtSpec {
                family: ArtFamily::Landmarks,
                stem: AssetStem::for_landmark(landmark.name.as_str()),
                pos,
                height: Millimeters::new(height_mm),
                anchor: AnchorFrac::PIN_TIP,
                rotation: angle + RotationRad::new(FRAC_PI_2),
                alpha: Alpha::OPAQUE,
            };
            if !state.assets.draw(ctx, state.style, &spec, px_per_mm) {
                renderer.draw_landmark(ctx, pos, state.params.landmark_scale, px_per_mm);
            }
        }
    }

    // Resolve every metro/underground edge's station path once
    // (metro: via > street mesh > chord, tunnels: via > chord) and
    // collect who shares each hop: metro by distinct line,
    // underground by edge - shared hops draw as parallel offsets
    // instead of piling identical strokes on top of each other.
    let mut routed_paths: Vec<(usize, EdgePath)> = Vec::new();
    let mut sharing: Sharing<Vec<HopShare>> = Sharing::default();
    for (edge_i, edge) in map.edges().iter().enumerate() {
        if !matches!(edge.transport, Transport::Metro | Transport::Underground) || line_hidden(edge) || !state.layers.transports.visible(edge.transport) {
            continue;
        }
        let path = map.edge_path(edge);
        let share = HopShare::of(edge, edge_i);
        for w in path.hops() {
            let bucket = sharing.entry(w).or_default();
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
    let mut surface_sharing: Sharing<BTreeSet<ParallelShare>> = Sharing::default();
    if state.layers.parallel_transports {
        for edge in map.edges() {
            if !matches!(edge.transport, Transport::Taxi | Transport::Bus) {
                continue;
            }
            let ids: Vec<StationId> = edge.station_chain().collect();
            for hop in ids.windows(2) {
                let key = StationPair::new(hop[0], hop[1]);
                surface_sharing.entry(key).or_default().insert(ParallelShare::Transport(edge.transport));
            }
        }
        for (edge_i, path) in &routed_paths {
            let Some(edge) = map.edges().get(*edge_i).filter(|edge| edge.is_transport(Transport::Metro)) else {
                continue;
            };
            let share = ParallelShare::for_metro(edge.line.as_ref(), metro_overlay);
            for hop in path.hops() {
                surface_sharing.entry(hop).or_default().insert(share.clone());
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
        let direction = b.pos - a.pos;
        (direction.magnitude() > Millimeters::new(1e-9)).then(|| direction.normalized().perpendicular())
    };
    // One continuity vote per via vertex: side(v) should equal
    // side(u) * c, with c = sign(rank_u * rank_v * (n_u . n_v)).
    let mut constraints: Vec<LaneConstraint> = Vec::new();
    if state.layers.parallel_transports {
        for edge in map.edges() {
            if !matches!(edge.transport, Transport::Taxi | Transport::Bus) {
                continue;
            }
            let hops: Vec<StationPair> = edge.chain().hops().collect();
            let share = ParallelShare::Transport(edge.transport);
            constraints.extend(LaneConstraint::chain(&hops, &surface_sharing.ranks(&hops, &share), hop_normal));
        }
        for (edge_i, path) in &routed_paths {
            let Some(edge) = map.edges().get(*edge_i).filter(|edge| edge.is_transport(Transport::Metro)) else {
                continue;
            };
            let share = ParallelShare::for_metro(edge.line.as_ref(), metro_overlay);
            let hops: Vec<StationPair> = path.hops().collect();
            constraints.extend(LaneConstraint::chain(&hops, &surface_sharing.ranks(&hops, &share), hop_normal));
        }
    } else {
        for (edge_i, path) in &routed_paths {
            let hops: Vec<StationPair> = path.hops().collect();
            let Some(edge) = map.edges().get(*edge_i) else { continue };
            let share = HopShare::of(edge, *edge_i);
            constraints.extend(LaneConstraint::chain(&hops, &sharing.ranks(&hops, &share), hop_normal));
        }
    }
    let hop_side = LaneSides::solve(&constraints);
    // The hop's parallel offset for a lane: its rank among the
    // lanes serving the hop, centered - relative to the canonical
    // a < b direction, so opposite-direction traversals land on
    // different lanes instead of collapsing onto one - times the
    // hop's lane side (keeps lanes on their side at via vertices).
    let spacing = px_per_mm.scale(Millimeters::new(2.6)).as_f64();
    let surface_offset = |share: &ParallelShare, a: StationId, b: StationId| -> f64 { share.offset(&surface_sharing, &hop_side, a, b, spacing) };
    let metro_in_parallel = |edge: &Edge| edge.is_transport(Transport::Metro) && state.layers.parallel_transports && !metro_overlay;

    // Metro + underground draw under the surface traffic - the
    // boats and roads ride over the tunnels.
    let edge_route = |edge_i: usize, edge: &Edge| -> Option<RoutePlan> {
        let (Some(_), Some(_)) = (map.station(edge.from), map.station(edge.to)) else {
            return None;
        };
        // `via` waypoints draw any edge's route as a bent polyline
        // (metro/tunnel hops shared with others draw parallel
        // offsets). Batobus edges already drew under the bridges.
        if matches!(edge.transport, Transport::Metro | Transport::Underground) {
            let path = routed_paths
                .iter()
                .find(|(e_i, _)| *e_i == edge_i)
                .map(|(_, path)| path.as_slice())
                .unwrap_or(&[]);
            let share = HopShare::of(edge, edge_i);
            let offsets: Vec<Pixels> = if edge.is_transport(Transport::Metro) && state.layers.parallel_transports {
                let share = ParallelShare::for_metro(edge.line.as_ref(), metro_overlay);
                path.windows(2).map(|w| Pixels::new(surface_offset(&share, w[0], w[1]))).collect()
            } else {
                path.windows(2)
                    .map(|w| {
                        let key = StationPair::new(w[0], w[1]);
                        let dir = key.direction(w[0]);
                        let side = hop_side.of(&key);
                        Pixels::new(sharing.offset(&key, &share, spacing, dir, side))
                    })
                    .collect()
            };
            let route_mm: Polyline = path.iter().filter_map(|id| map.station(*id)).map(|s| s.pos).collect();
            let route: Vec<PxPos> = route_mm.iter().map(|pos| to_px(*pos)).collect();
            Some(RoutePlan { route, offsets, route_mm })
        } else {
            // Above-ground edges dock at bridge-station deck ends;
            // everything else follows its `via` stations (or the
            // straight chord when `via` is empty). Hops shared by
            // several transports draw parallel offsets in the
            // edge's own color (parallel-transports overlay).
            let ids: Vec<StationId> = edge.station_chain().collect();
            let route_plan = map.road_route(&ids, decks, water_segments);
            let route_mm = &route_plan.polyline;
            let route: Vec<PxPos> = route_mm.iter().map(|pos| to_px(*pos)).collect();
            let offsets: Vec<Pixels> = if state.layers.parallel_transports {
                (0..route.len().saturating_sub(1))
                    .map(|segment| {
                        let hop = route_plan.hops[segment];
                        Pixels::new(surface_offset(&ParallelShare::Transport(edge.transport), hop.a(), hop.b()))
                    })
                    .collect()
            } else {
                Vec::new()
            };
            Some(RoutePlan {
                route,
                offsets,
                route_mm: route_plan.polyline,
            })
        }
    };
    let draw_surface_edge = |edge_i: usize, edge: &Edge| {
        let (Some(from), Some(to)) = (map.station(edge.from), map.station(edge.to)) else {
            return;
        };
        let Some(RoutePlan { route, offsets, .. }) = edge_route(edge_i, edge) else {
            return;
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
                wave_phase: Millimeters::ZERO,
                // A metro lane dashes while both overlays run, so
                // a line color matching a neighboring transport
                // lane stays readable.
                dashed: edge.is_transport(Transport::Metro) && state.layers.parallel_transports && metro_overlay,
                highlight: state.layers.highlight.contains(&edge.transport)
                    || (state.layers.parallel_transports && matches!(edge.transport, Transport::Taxi | Transport::Bus))
                    || metro_in_parallel(edge),
                px_per_mm,
            },
        );
    };

    // Dim regions for the tunnel lines: the river bands and every
    // park blob - where a metro/underground line crosses one, it
    // fades under a paper wash.
    let mut dim_regions: Vec<Polygon> = Vec::new();
    for course in water_courses {
        let banks = qdo36_map::WaterBanks::new(course, WATER_HALF_WIDTH);
        if banks.left.len() >= 2 {
            let mut band = banks.left;
            band.0.extend(banks.right.iter().rev().copied());
            dim_regions.push(Polygon::new(band.into()));
        }
    }
    if let Some(geometry) = &blocks_geometry {
        for block in &geometry.shapes {
            for park in &block.parks {
                dim_regions.push(park.polygon.clone());
            }
        }
    }
    for (edge_i, edge) in map.edges().iter().enumerate() {
        if !matches!(edge.transport, Transport::Metro | Transport::Underground) || !state.layers.transports.visible(edge.transport) || line_hidden(edge) {
            continue;
        }
        let Some(RoutePlan { route, route_mm, .. }) = edge_route(edge_i, edge) else {
            continue;
        };
        draw_surface_edge(edge_i, edge);
        // The tunnel fades inside every dim region it crosses.
        for region in &dim_regions {
            let crosses = route_mm.windows(2).any(|hop| region.crossed(&Segment::new(hop[0], hop[1])));
            if !crosses {
                continue;
            }
            let region_px: Vec<PxPos> = region.iter().map(|pos| to_px(*pos)).collect();
            ctx.save().ok();
            ctx.px_polyline_path(&region_px);
            ctx.close_path();
            ctx.clip();
            renderer.paper().apply_alpha(ctx, Alpha::new(0.55));
            ctx.set_line_width(px_per_mm.scale(Millimeters::new(2.6)).max(Pixels::new(0.5)).as_f64());
            ctx.px_polyline_path(&route);
            ctx.stroke().ok();
            ctx.restore().ok();
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
        let WaterRoute {
            route: mut route_mm,
            entry_arc,
        } = if edge.via.is_empty() {
            match (dock(edge.from), dock(edge.to)) {
                (Some(a), Some(b)) => WaterRoute::along(water_courses, a, b),
                _ => WaterRoute {
                    route: Polyline::new(vec![from.pos, to.pos]),
                    entry_arc: Millimeters::ZERO,
                },
            }
        } else {
            WaterRoute {
                route: edge.station_chain().filter_map(dock).collect(),
                entry_arc: Millimeters::ZERO,
            }
        };
        // The wave phase anchored to the course: the pier stub
        // shifts the local arc, so edges sharing a water stretch
        // trace the same wave instead of two crossing ones.
        let wave_phase = match (route_mm.first(), route_mm.get(1)) {
            (Some(a), Some(b)) => entry_arc - Millimeters::new((b.x - a.x).hypot(b.y - a.y)),
            _ => Millimeters::ZERO,
        };
        // Bridgehead docks bend into an arc: the water-course stub
        // inside the bend zone is trimmed, then two waypoints -
        // upstream of the deck center, part way toward the bank
        // end - turn the line in over open water before it slips
        // under the bridge.
        if route_mm.len() >= 2
            && let Some(deck) = decks.get(&edge.from).filter(|d| !d.on_water)
        {
            deck.dock_start(&mut route_mm);
        }
        if route_mm.len() >= 2
            && let Some(deck) = decks.get(&edge.to).filter(|d| !d.on_water)
        {
            deck.dock_end(&mut route_mm);
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
                wave_phase,
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
            let hop_ids: Vec<StationId> = edge.station_chain().collect();
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
                let Some(cross) = water_segments.iter().find_map(|water| road.crossing_point(water)) else {
                    continue;
                };
                let direction = road.to - road.from;
                if direction.magnitude() <= Millimeters::new(1e-9) {
                    continue;
                }
                let unit = direction.normalized();
                renderer.draw_bridge(
                    ctx,
                    to_px(cross - unit * BRIDGE_HALF_SPAN.as_f64()),
                    to_px(cross + unit * BRIDGE_HALF_SPAN.as_f64()),
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
    let served_transports = map.served_transports();

    // Taxi + bus draw over the tunnels and the boat courses -
    // surface traffic rides on top.
    for (edge_i, edge) in map.edges().iter().enumerate() {
        if !matches!(edge.transport, Transport::Taxi | Transport::Bus) || !state.layers.transports.visible(edge.transport) || line_hidden(edge) {
            continue;
        }
        draw_surface_edge(edge_i, edge);
    }

    // Per-station served metro lines drive the overlay pie fill.
    let served_lines = map.served_lines(&state.layers.hidden_lines);
    // Metro line badges on the routed paths: on the two edges
    // carrying the line's end stations plus every third segment.
    if metro_overlay {
        let mut lines: BTreeMap<LineName, Vec<EdgePath>> = BTreeMap::new();
        for (edge_i, path) in &routed_paths {
            if let Some(line) = map.edges().get(*edge_i).and_then(|e| e.line.clone()) {
                lines.entry(line).or_default().push(path.clone());
            }
        }
        for (line, paths) in &lines {
            // Chain the line's paths into one station sequence by
            // shared endpoints.
            let chain = EdgePath::joined(paths.clone());
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
                    let side = hop_side.of(&key);
                    sharing.offset(&key, &HopShare::Metro(line.clone()), spacing, dir, side)
                };
                let (a_px, b_px) = (to_px(a.pos), to_px(b.pos));
                let normal = (b_px - a_px).normalized().perpendicular();
                let mid = (a_px + b_px) * 0.5 + normal * offset;
                let spec = ArtSpec {
                    family: ArtFamily::Ligne,
                    stem: AssetStem::for_line(line),
                    pos: mid,
                    height: LIGNE_BADGE_HEIGHT * state.params.metro_scale.get(),
                    anchor: AnchorFrac::CENTER,
                    rotation: RotationRad::new(0.0),
                    alpha: Alpha::OPAQUE,
                };
                state.assets.draw(ctx, state.style, &spec, px_per_mm);
            }
        }
    }

    // Stations + labels + numbers + scores. Radial labels face the
    // nearest *screen* edge - not the map center (which pans off-
    // viewport) - the same `center` the radial landmarks use.
    let importance = map.importance();
    let orientation = state.layers.label_mode.orientation(center);
    let radius = px_per_mm.scale(STATION_RADIUS_MM).max(Pixels::new(0.4));
    for station in map.stations() {
        let pos = to_px(station.pos);
        let station_importance = importance.get(&station.id);
        if state.layers.stations {
            renderer.draw_station(
                ctx,
                &StationSpec {
                    pos,
                    served: served_transports.get(&station.id).map(Vec::as_slice).unwrap_or_default(),
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
                    radius: radius * 2.4,
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
        for station in map.stations_of_kind(StationKind::Bridge) {
            renderer.draw_landmark(ctx, to_px(station.pos), ScaleFactor::new(2.0), px_per_mm);
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
        height: ANCHOR_HEIGHT,
        anchor: AnchorFrac::PIN_TIP,
        rotation: RotationRad::new(0.0),
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
            height: COMPASS_HEIGHT,
            anchor: AnchorFrac::CENTER,
            rotation: RotationRad::new(0.0),
            alpha: Alpha::new(0.9),
        };
        state.assets.draw(ctx, state.style, &compass_spec, px_per_mm);
    }

    // "Tower missing" hint - only while virtual; floats above
    // the menu strip at the screen edge.
    if let (Some(hint), AnchorState::Virtual(_)) = (&state.anchor_hint, state.anchor) {
        renderer.draw_hint(ctx, hint, width, height, Pixels::new(controls::EDGE_RESERVE));
    }
}

/// The station obstacle discs the landmark pins dodge - position
/// and radius in px (the radius pads the nominal station disc by
/// ~10 %).
fn station_discs(state: &SceneState, stage: &RenderStage) -> Vec<(PxPos, Pixels)> {
    state
        .map
        .stations()
        .map(|station| {
            let radius = STATION_RADIUS_MM * (1.1 * state.params.station_scale.get());
            (stage.to_px(station.pos), stage.fit.scale.scale(radius))
        })
        .collect()
}

/// The pin direction (screen radians) a landmark artwork takes:
/// the candidate direction least covered by station discs and
/// edges, staying closest to `preferred` when ties allow. The pin
/// body sits `body_offset` px out from the tip and covers
/// `body_radius`.
fn landmark_rotation(
    pos: PxPos,
    preferred: RotationRad,
    body_offset: f64,
    body_radius: f64,
    stations: &[(PxPos, Pixels)],
    edges: &[(PxPos, PxPos)],
) -> RotationRad {
    const STEPS: usize = 16;
    let segment_distance = |px: f64, py: f64, a: PxPos, b: PxPos| {
        let (vx, vy) = (b.x - a.x, b.y - a.y);
        let len_sq = vx * vx + vy * vy;
        let t = if len_sq > 1e-9 {
            (((px - a.x) * vx + (py - a.y) * vy) / len_sq).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (px - a.x - t * vx).hypot(py - a.y - t * vy)
    };
    let mut best = (f64::INFINITY, f64::INFINITY, preferred);
    for i in 0..STEPS {
        let angle = RotationRad::new(i as f64 * TAU / STEPS as f64);
        let (cx, cy) = (pos.x + angle.as_f64().cos() * body_offset, pos.y + angle.as_f64().sin() * body_offset);
        let station_cover: f64 = stations
            .iter()
            .map(|(s, r)| (r.as_f64() + body_radius - (cx - s.x).hypot(cy - s.y)).max(0.0))
            .sum();
        let edge_cover: f64 = edges.iter().map(|(a, b)| (body_radius - segment_distance(cx, cy, *a, *b)).max(0.0)).sum();
        let occlusion = station_cover + edge_cover;
        let deviation = angle.signed_delta_to(preferred).as_f64().abs();
        if occlusion < best.0 - 1e-9 || (occlusion <= best.0 + 1e-9 && deviation < best.1) {
            best = (occlusion, deviation, angle);
        }
    }
    best.2
}
