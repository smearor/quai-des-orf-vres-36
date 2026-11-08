//! `BlocksGeometry` - the mode-dependent block render data.

use cairo::Context;
use qdo36_map::BlockCut;
use qdo36_map::BlockLayout;
use qdo36_map::BuildingParams;
use qdo36_map::Depth;
use qdo36_map::Ground;
use qdo36_map::Height;
use qdo36_map::House;
use qdo36_map::Obstacles;
use qdo36_map::Polygon;
use qdo36_map::Segment;
use qdo36_map::Street;
use qdo36_map::Units;
use qdo36_map::Variant;
use std::rc::Rc;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PixelsPerMm;

use crate::ui::map_scene::SceneLayers;
use crate::ui::map_scene::blocks::BlockShapes;
use crate::ui::style::BlockSpec;
use crate::ui::style::DistrictLabel;
use crate::ui::style::ParkSpot;
use crate::ui::style::PxPos;
use crate::ui::style::StyleRenderer;
use crate::ui::style::ToPx;

/// The mode-dependent block render data: drawable inner blocks
/// plus the ground mask covering the whole city - everything
/// outside it takes the style's outer texture instead.
pub struct BlocksGeometry {
    /// Drawable blocks: inner faces with a valid inset, without
    /// the faces underground lines suppress.
    pub shapes: Vec<BlockShapes>,
    /// Ground mask covering the city: every inner face's raw
    /// outline expanded by each edge's setback, so the fill reaches
    /// exactly the outer faces' inset edge - streets, corridors
    /// and suppressed faces all rest on ground, never on texture.
    pub ground: Ground,
}

/// The cache key for the mode-dependent block geometry.
pub fn blocks_key(parallel: bool, metro_overlay: bool) -> u8 {
    u8::from(parallel) | (u8::from(metro_overlay) << 1)
}

impl BlocksGeometry {
    /// Computes every face's inset outline, houses and the city
    /// ground mask for the current view mode.
    pub fn compute(layout: &BlockLayout, parallel: bool, metro_overlay: bool) -> Self {
        let margin = |cut: &BlockCut| layout.setback(cut, parallel, metro_overlay).as_f64();
        let params = BuildingParams::default();
        let mut shapes = Vec::new();
        let mut ground = Ground::default();
        for (index, face) in layout.faces.iter().enumerate() {
            if face.is_outer() {
                continue;
            }
            ground.0.push(face.outset(&margin));
            if let Some(outline) = face.inset(&margin) {
                // Obstacles the houses stay out of: tunnel corridors,
                // decorative street corridors, park blobs.
                let mut obstacles = Obstacles(face.tunnels.iter().map(|tunnel| tunnel.corridor(Millimeters::new(0.8))).collect());
                let streets = Street::of_block(&outline, index as u64);
                for street in &streets {
                    for pair in street.windows(2) {
                        obstacles.0.push(Segment::new(pair[0], pair[1]).corridor(Millimeters::new(1.6)));
                    }
                }
                let parks: Vec<ParkSpot> = face
                    .parks
                    .iter()
                    .filter_map(|park| {
                        park.polygon(&outline, index as u64).map(|polygon| ParkSpot {
                            name: park.name.clone(),
                            polygon,
                        })
                    })
                    .collect();
                for park in &parks {
                    obstacles.0.push(park.polygon.clone());
                }
                // Landmark volumes: a landmark inside the block becomes
                // a tall octagonal house - its pin stays the roof plate.
                let landmark_footprints: Vec<Polygon> = face.landmarks.iter().filter_map(|landmark| landmark.footprint(&outline)).collect();
                obstacles.0.extend(landmark_footprints.iter().cloned());
                // The district caption dodges the park blobs so it
                // never lands on a park name.
                let avoid: Vec<Polygon> = parks.iter().map(|park| park.polygon.clone()).collect();
                let label = face.district.as_ref().and_then(|name| {
                    outline.deepest_point_avoiding(&avoid).map(|pos| DistrictLabel {
                        name: Rc::from(name.as_str()),
                        pos,
                    })
                });
                let mut buildings = params.houses(&outline, index as u64, &obstacles);
                buildings.extend(landmark_footprints.into_iter().map(|footprint| House {
                    footprint,
                    units: Units::ONE,
                    variant: Variant::LANDMARK,
                    depth: Depth::LANDMARK,
                    height: Height::LANDMARK,
                }));
                shapes.push(BlockShapes {
                    buildings,
                    outline,
                    streets: streets.into_iter().map(|street| street.0).collect(),
                    parks,
                    label,
                });
            }
        }
        Self { shapes, ground }
    }

    /// Draws every inner block's inset outline and houses. `layers`
    /// picks the caption orientation per district/park label;
    /// `center` (widget px) is the radial pivot.
    pub fn draw(&self, ctx: &Context, renderer: &dyn StyleRenderer, layers: &SceneLayers, center: PxPos, px_per_mm: PixelsPerMm, to_px: ToPx<'_>) {
        for block in &self.shapes {
            renderer.draw_block(
                ctx,
                &BlockSpec {
                    outline: &block.outline,
                    buildings: &block.buildings,
                    streets: &block.streets,
                    parks: &block.parks,
                    label: block.label.clone(),
                    district_orientation: layers.district_mode.orientation(center),
                    park_orientation: layers.park_mode.orientation(center),
                    house_mode: layers.house_mode,
                    center,
                    px_per_mm,
                },
                to_px,
            );
        }
    }
}
