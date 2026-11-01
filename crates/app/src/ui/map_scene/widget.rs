//! `MapScene` - the GTK widget rendering the map scene.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::prelude::WidgetExt;
use gtk4::prelude::WidgetExtManual;
use qdo36_map::StationId;
use qdo36_tangibles::AnchorState;
use tangible_objects_model::Pixels;

use crate::ui::map_scene::PoseAnimator;
use crate::ui::map_scene::RenderStage;
use crate::ui::map_scene::SceneState;
use crate::ui::style::PxPos;

/// Snap radius for tap hit-testing, in map millimeters.
const SNAP_RADIUS_MM: f64 = 12.0;

mod imp {
    use super::*;
    use gtk4::subclass::prelude::*;

    /// State held by the widget.
    pub struct MapSceneImp {
        /// The current scene state.
        pub state: RefCell<Option<Rc<SceneState>>>,
        /// Interpolates the rendered anchor pose toward the target in
        /// `state` - survives `update()` replacing the state.
        pub animator: RefCell<PoseAnimator>,
    }

    impl Default for MapSceneImp {
        fn default() -> Self {
            Self {
                state: RefCell::new(None),
                animator: RefCell::new(PoseAnimator::new(AnchorState::default_virtual().pose(), Instant::now())),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MapSceneImp {
        const NAME: &'static str = "Qdo36MapScene";
        type Type = super::MapScene;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for MapSceneImp {}

    impl WidgetImpl for MapSceneImp {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let widget = self.obj();
            let Some(state) = self.state.borrow().clone() else {
                return;
            };
            let width = widget.width() as f64;
            let height = widget.height() as f64;
            if width <= 0.0 || height <= 0.0 {
                return;
            }
            let rect = gtk4::graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
            let ctx = gtk4::prelude::SnapshotExt::append_cairo(snapshot, &rect);
            let pose = self.animator.borrow_mut().pose_at(Instant::now());
            super::super::render(&ctx, &state, pose, Pixels::new(width), Pixels::new(height));
        }
    }
}

glib::wrapper! {
    /// The map scene widget.
    pub struct MapScene(ObjectSubclass<imp::MapSceneImp>)
        @extends gtk4::Widget,
        @implements gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Accessible;
}

impl Default for MapScene {
    fn default() -> Self {
        Self::new()
    }
}

impl MapScene {
    /// Creates the scene widget.
    pub fn new() -> Self {
        let scene: Self = glib::Object::new();
        // Frame driver: redraw only while the pose animation runs.
        scene.add_tick_callback(|scene, _clock| {
            if scene.imp().animator.borrow().is_animating() {
                scene.queue_draw();
            }
            glib::ControlFlow::Continue
        });
        scene
    }

    /// Replaces the scene state, retargets the pose animation and
    /// queues a redraw.
    pub fn update(&self, state: Rc<SceneState>) {
        self.imp().animator.borrow_mut().set_target(state.anchor.pose(), Instant::now());
        *self.imp().state.borrow_mut() = Some(state);
        self.queue_draw();
    }

    /// The station nearest to a widget-px tap (snap radius ~12 mm).
    pub fn station_at(&self, state: &SceneState, pos: PxPos) -> Option<StationId> {
        let pose = self.imp().animator.borrow_mut().pose_at(Instant::now());
        let stage = RenderStage::new(state, pose, Pixels::new(self.width() as f64), Pixels::new(self.height() as f64));
        state
            .map
            .nearest_station(stage.to_map(pos))
            .filter(|(_, distance)| distance.as_f64() <= SNAP_RADIUS_MM)
            .map(|(id, _)| id)
    }
}
