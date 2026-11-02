//! The map scene widget: renders the GameMap under `MapTransform`
//! (map-mm -> surface-mm -> widget-px) on every snapshot.

pub mod pose_animator;
pub mod render_stage;
pub mod scene_state;
pub mod widget;

use cairo::Context;
pub use pose_animator::PoseAnimator;
pub use render_stage::RenderStage;
pub use scene_state::SceneState;
pub use widget::MapScene;

use qdo36_tangibles::AnchorPose;
use qdo36_tangibles::AnchorState;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Pixels;
use tangible_objects_model::Vec2;

use crate::ui::art::AnchorFrac;
use crate::ui::art::ArtFamily;
use crate::ui::art::ArtSpec;
use crate::ui::art::AssetStem;
use crate::ui::style::Alpha;
use crate::ui::style::EdgeSpec;

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
    for polyline in &backdrop.water {
        renderer.draw_water(ctx, polyline.points(), px_per_mm, &to_px);
    }
    for landmark in &backdrop.landmarks {
        let pos = to_px(landmark.pos);
        let spec = ArtSpec {
            family: ArtFamily::Landmarks,
            stem: AssetStem::for_landmark(landmark.name.as_str()),
            pos,
            height: Millimeters::new(14.0),
            anchor: AnchorFrac::PIN_TIP,
            alpha: Alpha::OPAQUE,
        };
        if !state.assets.draw(ctx, state.style, &spec, px_per_mm) {
            renderer.draw_landmark(ctx, pos, px_per_mm);
        }
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
    let bounds = state.map.bounds();
    let compass_pos = to_px(Vec2::new(bounds.x_max.as_f64() + 16.0, bounds.y_min.as_f64() - 16.0));
    let compass_spec = ArtSpec {
        family: ArtFamily::Map,
        stem: AssetStem::new("compass"),
        pos: compass_pos,
        height: Millimeters::new(26.0),
        anchor: AnchorFrac::CENTER,
        alpha: Alpha::new(0.9),
    };
    state.assets.draw(ctx, state.style, &compass_spec, px_per_mm);

    // "Tower missing" hint - only while virtual.
    if let (Some(hint), AnchorState::Virtual(_)) = (&state.anchor_hint, state.anchor) {
        renderer.draw_hint(ctx, hint, width, height);
    }
}
