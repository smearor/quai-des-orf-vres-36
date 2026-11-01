//! The map scene widget: renders the GameMap under `MapTransform`
//! (map-mm -> surface-mm -> widget-px) on every snapshot.

pub mod render_stage;
pub mod scene_state;
pub mod widget;

pub use render_stage::RenderStage;
pub use scene_state::SceneState;
pub use widget::MapScene;

use qdo36_tangibles::AnchorState;
use tangible_objects_model::Pixels;

use crate::ui::style::EdgeSpec;

/// Full scene render: paper, backdrop, edges, stations, anchor,
/// overlay - each element delegated to the active `StyleRenderer`.
pub fn render(ctx: &cairo::Context, state: &SceneState, width: f64, height: f64) {
    let map = state.map.as_ref();
    let stage = RenderStage::new(state, width, height);
    let to_px = |pos| stage.to_px(pos);
    let px_per_mm = stage.fit.scale;
    let renderer = state.style.renderer();

    // Paper.
    renderer.paper().apply(ctx);
    ctx.paint().ok();

    // Backdrop: water + landmarks (gameplay-inert dressing).
    let backdrop = map.backdrop();
    for polyline in &backdrop.water {
        renderer.draw_water(ctx, polyline.points(), px_per_mm, &to_px);
    }
    for landmark in &backdrop.landmarks {
        renderer.draw_landmark(ctx, to_px(landmark.pos), px_per_mm);
    }

    // Edges.
    for edge in map.edges() {
        let (Some(from), Some(to)) = (map.station(edge.from), map.station(edge.to)) else {
            continue;
        };
        renderer.draw_edge(
            ctx,
            &EdgeSpec {
                from: to_px(from.pos),
                to: to_px(to.pos),
                transport: edge.transport,
                line: edge.line.as_ref(),
                map,
                px_per_mm,
            },
        );
    }

    // Stations + labels.
    for station in map.stations() {
        let pos = to_px(station.pos);
        renderer.draw_station(ctx, pos, state.selected == Some(station.id), px_per_mm);
        let radius = Pixels::new(px_per_mm.scale(3.0).max(0.4));
        renderer.draw_label(ctx, station.name.as_str(), pos, radius, px_per_mm);
    }

    // Anchor marker.
    renderer.draw_anchor(ctx, to_px(state.config.anchor_map_pos), matches!(state.anchor, AnchorState::Tracked(_)), px_per_mm);

    // "Tower missing" hint - only while virtual.
    if let (Some(hint), AnchorState::Virtual(_)) = (&state.anchor_hint, state.anchor) {
        renderer.draw_hint(ctx, hint, Pixels::new(width), Pixels::new(height));
    }
}
