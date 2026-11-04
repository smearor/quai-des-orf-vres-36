//! The map scene widget: renders the GameMap under `MapTransform`
//! (map-mm -> surface-mm -> widget-px) on every snapshot.

pub mod controls;
pub mod label_mode;
pub mod layer_name;
pub mod layer_toggle;
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
pub use label_mode::LabelMode;
pub use layer_name::LayerName;
pub use layer_toggle::LayerToggle;
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
use crate::ui::style::Alpha;
use crate::ui::style::EdgeSpec;
use crate::ui::style::LabelOrientation;
use crate::ui::style::LabelSpec;
use crate::ui::style::PxPos;
use crate::ui::style::StationSpec;
use qdo36_map::Edge;
use qdo36_map::LineName;
use qdo36_map::Segment;
use qdo36_map::StationId;
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

    // Backdrop: water + landmarks (gameplay-inert dressing).
    let backdrop = map.backdrop();
    if state.layers.water {
        for polyline in &backdrop.water {
            renderer.draw_water(ctx, polyline.points(), px_per_mm, &to_px);
        }
    }
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

    // Bridge decks: where a visible taxi/bus edge crosses the
    // water, a pale plank under the road line (decorative).
    if state.layers.water {
        let water_segments: Vec<Segment<Millimeters>> = backdrop
            .water
            .iter()
            .flat_map(|polyline| polyline.points().windows(2).map(|pair| Segment::new(pair[0], pair[1])))
            .collect();
        for edge in map.edges() {
            if !matches!(edge.transport, Transport::Taxi | Transport::Bus) || !state.layers.transports.visible(edge.transport) {
                continue;
            }
            let (Some(from), Some(to)) = (map.station(edge.from), map.station(edge.to)) else {
                continue;
            };
            // Strict crossing only - an edge merely ending on the
            // waterline is a pier access, not a bridge (Q13 rule).
            let road = Segment::new(from.pos, to.pos);
            let Some(cross) = water_segments
                .iter()
                .find_map(|water| if road.crosses(water) { road.intersection(water) } else { None })
            else {
                continue;
            };
            let (dx, dy) = (to.pos.x - from.pos.x, to.pos.y - from.pos.y);
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
    let line_hidden = |edge: &Edge| edge.line.as_ref().is_some_and(|line| state.layers.hidden_lines.contains(line));

    // Resolve every metro edge's station path once (via > street
    // > chord) and collect the distinct lines sharing each hop.
    let mut metro_paths: Vec<(usize, Vec<StationId>)> = Vec::new();
    let mut sharing: HashMap<(StationId, StationId), BTreeSet<Option<LineName>>> = HashMap::new();
    for (edge_i, edge) in map.edges().iter().enumerate() {
        if edge.transport != Transport::Metro || line_hidden(edge) || !state.layers.transports.visible(edge.transport) {
            continue;
        }
        let path = if !edge.via.is_empty() {
            let mut path = vec![edge.from];
            path.extend_from_slice(&edge.via);
            path.push(edge.to);
            path
        } else if edge.direct {
            vec![edge.from, edge.to]
        } else {
            map.route_on_surface(edge.from, edge.to).unwrap_or_else(|| vec![edge.from, edge.to])
        };
        for w in path.windows(2) {
            let key = if w[0] < w[1] { (w[0], w[1]) } else { (w[1], w[0]) };
            sharing.entry(key).or_default().insert(edge.line.clone());
        }
        metro_paths.push((edge_i, path));
    }

    for (edge_i, edge) in map.edges().iter().enumerate() {
        if !state.layers.transports.visible(edge.transport) || line_hidden(edge) {
            continue;
        }
        let (Some(from), Some(to)) = (map.station(edge.from), map.station(edge.to)) else {
            continue;
        };
        // `via` waypoints draw any edge's route as a bent polyline
        // (metro additionally offsets hops shared by other lines).
        let (route, offsets) = if edge.transport == Transport::Metro {
            let path = metro_paths
                .iter()
                .find(|(e_i, _)| *e_i == edge_i)
                .map(|(_, path)| path.as_slice())
                .unwrap_or(&[]);
            let spacing = px_per_mm.scale(1.3);
            let offsets: Vec<f64> = path
                .windows(2)
                .map(|w| {
                    let key = if w[0] < w[1] { (w[0], w[1]) } else { (w[1], w[0]) };
                    let (count, rank) = sharing
                        .get(&key)
                        .map(|lines| (lines.len(), lines.iter().position(|l| *l == edge.line).unwrap_or(0)))
                        .unwrap_or((1, 0));
                    (rank as f64 - (count as f64 - 1.0) / 2.0) * spacing
                })
                .collect();
            let route: Vec<_> = path.iter().filter_map(|id| map.station(*id)).map(|s| to_px(s.pos)).collect();
            (route, offsets)
        } else if !edge.via.is_empty() {
            let route: Vec<PxPos> = [edge.from]
                .into_iter()
                .chain(edge.via.iter().copied())
                .chain([edge.to])
                .filter_map(|id| map.station(id))
                .map(|s| to_px(s.pos))
                .collect();
            (route, Vec::new())
        } else {
            (Vec::new(), Vec::new())
        };
        renderer.draw_edge(
            ctx,
            &EdgeSpec {
                from: to_px(from.pos),
                to: to_px(to.pos),
                transport: edge.transport,
                line: edge.line.as_ref(),
                map,
                route: if route.len() > 2 { route } else { Vec::new() },
                offsets,
                highlight: state.layers.highlight.contains(&edge.transport),
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
    let metro_overlay = state.layers.highlight.contains(&Transport::Metro);

    // Metro line badges on the routed paths: on the two edges
    // carrying the line's end stations plus every third segment.
    if metro_overlay {
        let mut lines: BTreeMap<LineName, Vec<Vec<StationId>>> = BTreeMap::new();
        for (edge_i, path) in &metro_paths {
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
                let mid = Segment::new(to_px(a.pos), to_px(b.pos)).midpoint();
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
