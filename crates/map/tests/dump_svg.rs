//! Debug dump: renders the bundled map's block layout to SVGs for
//! visual inspection:
//!
//! ```sh
//! cargo test -p qdo36-map --test dump_svg dump -- --nocapture
//! rsvg-convert -w 1600 /tmp/blocks.svg -o /tmp/blocks.png
//! ```
//!
//! `dump_zooms` additionally writes a close-up per review case
//! (stations sets) into `/tmp/zoom_<name>.svg`.

use qdo36_map::BlockCut;
use qdo36_map::BlockLayout;
use qdo36_map::GameMap;
use qdo36_map::Obstacles;
use qdo36_map::Polygon;
use qdo36_map::Segment;
use qdo36_map::StationId;
use qdo36_map::Street;
use qdo36_map::Transport;
use std::fmt::Write;
use tangible_objects_model::Millimeters;

/// The drawn river band's half width (matches the app's
/// `WATER_HALF_WIDTH`).
const RIVER_HALF: Millimeters = Millimeters::new(3.5);

/// The setback the app keeps from each cut kind (widest corridor:
/// parallel view with the metro overlay split).
fn margin(layout: &BlockLayout) -> impl Fn(&BlockCut) -> f64 + '_ {
    move |cut: &BlockCut| layout.setback(cut, true, true).as_f64()
}

/// Writes the block layout SVG to `path`. `focus` crops the
/// viewBox around the given stations (plus 25 mm context); empty
/// keeps the whole map.
fn write_svg(map: &GameMap, layout: &BlockLayout, path: &str, focus: &[StationId]) {
    let bounds = map.bounds();
    let (vx, vy, vw, vh) = if focus.is_empty() {
        (bounds.x_min.as_f64(), bounds.y_min.as_f64(), bounds.width().as_f64(), bounds.height().as_f64())
    } else {
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for id in focus {
            if let Some(station) = map.station(*id) {
                min_x = min_x.min(station.pos.x);
                max_x = max_x.max(station.pos.x);
                min_y = min_y.min(station.pos.y);
                max_y = max_y.max(station.pos.y);
            }
        }
        (min_x - 25.0, min_y - 25.0, max_x - min_x + 50.0, max_y - min_y + 50.0)
    };
    let mut svg = String::new();
    let _ = writeln!(svg, r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vx} {vy} {vw} {vh}">"#);
    let _ = writeln!(svg, r##"<rect x="{vx}" y="{vy}" width="{vw}" height="{vh}" fill="#F7EBD0"/>"##);
    // water courses
    for course in map.smoothed_water() {
        let pts: Vec<String> = course.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
        let _ = writeln!(svg, r##"<polyline points="{}" fill="none" stroke="#2E7385" stroke-width="7" opacity="0.5"/>"##, pts.join(" "));
        let _ = writeln!(svg, r##"<polyline points="{}" fill="none" stroke="#226" stroke-width="0.6"/>"##, pts.join(" "));
    }
    // streets: the routed roads the renderer draws (deck ends,
    // bridgehead bends) - the same geometry the block cuts use.
    let water_segments: Vec<Segment<Millimeters>> = map.smoothed_water().iter().flat_map(|course| course.segments()).collect();
    let decks = map.bridge_decks(RIVER_HALF);
    for edge in map.edges() {
        if !matches!(edge.transport, Transport::Taxi | Transport::Bus) {
            continue;
        }
        let ids = map.edge_path(edge);
        let route = map.road_route(&ids, &decks, &water_segments).polyline;
        let pts: Vec<String> = route.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
        let color = if edge.transport == Transport::Taxi { "#c9a227" } else { "#386B48" };
        let _ = writeln!(svg, r##"<polyline points="{}" fill="none" stroke="{}" stroke-width="1.2"/>"##, pts.join(" "), color);
    }
    // underground tunnels (dashed red - they cut houses, not blocks)
    for edge in map.edges() {
        if edge.transport != Transport::Underground {
            continue;
        }
        let pts: Vec<String> = map
            .edge_path(edge)
            .iter()
            .filter_map(|id| map.station(*id))
            .map(|s| format!("{},{}", s.pos.x, s.pos.y))
            .collect();
        let _ = writeln!(
            svg,
            r##"<polyline points="{}" fill="none" stroke="#cc3333" stroke-width="0.8" stroke-dasharray="2.5 1.5"/>"##,
            pts.join(" ")
        );
    }
    // raw face outlines (thin red)
    for face in &layout.faces {
        let pts: Vec<String> = face.outline.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
        let _ = writeln!(
            svg,
            r##"<polygon points="{}" fill="none" stroke="#ff0000" stroke-width="0.4" opacity="0.4"/>"##,
            pts.join(" ")
        );
    }
    // inset outlines + decorative streets/parks + procedural
    // houses like the app draws them
    let margin = margin(layout);
    let params = qdo36_map::BuildingParams::default();
    let mut house_count = 0;
    for (index, face) in layout.faces.iter().enumerate() {
        if face.is_outer() {
            continue;
        }
        if let Some(outline) = face.inset(&margin) {
            // obstacles: tunnel corridors, decorative street
            // corridors, park blobs - the same pipeline the app runs.
            let mut obstacles_vec: Vec<Polygon> = face.tunnels.iter().map(|tunnel| tunnel.corridor(Millimeters::new(0.8))).collect();
            let streets = Street::of_block(&outline, index as u64);
            for street in &streets {
                for pair in street.windows(2) {
                    obstacles_vec.push(Segment::new(pair[0], pair[1]).corridor(Millimeters::new(1.6)));
                }
            }
            let park_blobs: Vec<(String, Polygon)> = face
                .parks
                .iter()
                .filter_map(|park| park.polygon(&outline, index as u64).map(|blob| (park.name.to_string(), blob)))
                .collect();
            for (_, blob) in &park_blobs {
                obstacles_vec.push(blob.clone());
            }
            // park blobs first (under houses)
            for (name, blob) in &park_blobs {
                let pts: Vec<String> = blob.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
                let _ = writeln!(
                    svg,
                    r##"<polygon points="{}" fill="#7A9E5A" fill-opacity="0.55" stroke="#476232" stroke-width="0.5"/>"##,
                    pts.join(" ")
                );
                let _ = writeln!(
                    svg,
                    r##"<text x="{}" y="{}" font-size="2.4" fill="#476232" text-anchor="middle" font-style="italic">{}</text>"##,
                    blob.centroid().x,
                    blob.centroid().y,
                    name
                );
            }
            // decorative streets
            for street in &streets {
                let pts: Vec<String> = street.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
                let _ = writeln!(
                    svg,
                    r##"<polyline points="{}" fill="none" stroke="#7A5B28" stroke-width="1.4" opacity="0.7"/>"##,
                    pts.join(" ")
                );
            }
            for house in params.houses(&outline, index as u64, &Obstacles(obstacles_vec.clone())) {
                house_count += 1;
                let pts: Vec<String> = house.footprint.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
                let shade = ["#E4D3A9", "#DCC79B", "#EEDCB5", "#D5C08F"][house.variant.as_u8() as usize % 4];
                let _ = writeln!(svg, r##"<polygon points="{}" fill="{}" stroke="#542C15" stroke-width="0.35"/>"##, pts.join(" "), shade);
                if house.footprint.len() == 4 {
                    // roof ridge between the depth-side midpoints
                    let (a, b) = (house.footprint[1], house.footprint[2]);
                    let (c, d) = (house.footprint[3], house.footprint[0]);
                    let _ = writeln!(
                        svg,
                        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#542C15" stroke-width="0.25"/>"##,
                        (a.x + b.x) / 2.0,
                        (a.y + b.y) / 2.0,
                        (c.x + d.x) / 2.0,
                        (c.y + d.y) / 2.0
                    );
                }
            }
            let pts: Vec<String> = outline.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
            let _ = writeln!(
                svg,
                r##"<polygon points="{}" fill="#8C734C" fill-opacity="0.15" stroke="#542C15" stroke-width="0.7"/>"##,
                pts.join(" ")
            );
            // district label at the deepest inside point
            if let (Some(name), Some(anchor)) = (face.district.as_ref().map(|name| name.as_str()), face.label_anchor()) {
                let _ = writeln!(
                    svg,
                    r##"<text x="{}" y="{}" font-size="4" fill="#542C15" fill-opacity="0.6" text-anchor="middle" font-style="italic">{}</text>"##,
                    anchor.x, anchor.y, name
                );
            }
        }
    }
    // stations
    for s in map.stations() {
        let _ = writeln!(
            svg,
            r##"<circle cx="{}" cy="{}" r="3" fill="#FAF2DB" stroke="#542C15" stroke-width="0.6"/>"##,
            s.pos.x, s.pos.y
        );
        let _ = writeln!(
            svg,
            r##"<text x="{}" y="{}" font-size="3" fill="#333" text-anchor="middle">{}</text>"##,
            s.pos.x,
            s.pos.y - 4.0,
            s.id
        );
    }
    svg.push_str("</svg>");
    std::fs::write(path, &svg).expect("write");
    eprintln!("{path}: houses {house_count}");
}

#[test]
fn dump_blocks_svg() {
    let map = GameMap::bundled().expect("paris");
    let layout = map.block_layout(RIVER_HALF);
    write_svg(&map, &layout, "/tmp/blocks.svg", &[]);
    eprintln!("faces: {}  lanes: {}", layout.faces.len(), layout.lanes.len());
}

/// Review close-ups: tiny triangle, bridgehead and collinear
/// stations.
#[test]
fn dump_zooms() {
    let map = GameMap::bundled().expect("paris");
    let layout = map.block_layout(RIVER_HALF);
    let cases: &[(&str, &[u16])] = &[
        ("few_houses", &[2, 35, 36]),
        ("bridgehead", &[1, 46, 49]),
        ("collinear", &[46, 75, 49, 2]),
        ("tiny_face", &[2, 49, 82]),
    ];
    for (name, ids) in cases {
        write_svg(&map, &layout, &format!("/tmp/zoom_{name}.svg"), &ids.iter().map(|id| StationId::new(*id)).collect::<Vec<_>>());
    }
}

#[test]
fn inset_failure_report() {
    let map = GameMap::bundled().expect("paris");
    let layout = map.block_layout(RIVER_HALF);
    let margin = margin(&layout);
    for (index, face) in layout.faces.iter().enumerate() {
        if face.inset(&margin).is_none() {
            let (x, y) = face
                .outline
                .iter()
                .fold((0.0, 0.0), |(sx, sy), p| (sx + p.x / face.outline.len() as f64, sy + p.y / face.outline.len() as f64));
            eprintln!("face {index}: area {:.1}, verts {}, centroid ({x:.0},{y:.0})", face.signed_area(), face.outline.len());
        }
    }
}

#[test]
fn parks_report() {
    let map = GameMap::bundled().expect("paris");
    let layout = map.block_layout(RIVER_HALF);
    let margin = margin(&layout);
    for (index, face) in layout.faces.iter().enumerate() {
        for park in &face.parks {
            let status = face
                .inset(&margin)
                .map(|outline| {
                    let anchor = outline.deepest_point().map(|p| format!("anchor ({:.0},{:.0})", p.x, p.y)).unwrap_or_default();
                    format!("inset ok, blob {}, {}", park.polygon(&outline, index as u64).is_some(), anchor)
                })
                .unwrap_or_else(|| "no inset".to_string());
            eprintln!("park {:?} at {:?}: face {index} -> {status}", park.name, (park.pos.x, park.pos.y));
        }
    }
}

#[test]
fn suppression_report() {
    let map = GameMap::bundled().expect("paris");
    let layout = map.block_layout(RIVER_HALF);
    let tunnel_faces = layout.faces.iter().filter(|f| !f.tunnels.is_empty()).count();
    let outer = layout.faces.iter().filter(|f| f.is_outer()).count();
    eprintln!("tunnel faces: {tunnel_faces}  outer: {outer}  total: {}", layout.faces.len());
    let underground = map.edges().iter().filter(|e| e.transport == Transport::Underground).count();
    eprintln!("underground edges: {underground}");
}
