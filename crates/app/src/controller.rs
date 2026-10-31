//! The app controller: owns the daemon source, the anchor state and
//! the event pump feeding the UI.

use std::cell::RefCell;
use std::rc::Rc;

use qdo36_tangibles::AnchorPose;
use qdo36_tangibles::AnchorState;
use qdo36_tangibles::GraphQlSource;
use qdo36_tangibles::TangibleEvent;
use tracing::warn;

use crate::tangible_host::TangibleHost;

// The active controller (single-window app).
thread_local! {
    static ACTIVE_CONTROLLER: RefCell<Option<Rc<RefCell<Controller>>>> = const { RefCell::new(None) };
}

/// Installs `controller` as the active instance.
pub fn set_active_controller(controller: Rc<RefCell<Controller>>) {
    ACTIVE_CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller));
}

/// Runs `f` with the active controller, when set.
pub fn with_controller(f: impl FnOnce(Rc<RefCell<Controller>>)) {
    ACTIVE_CONTROLLER.with(|slot| {
        if let Some(controller) = slot.borrow().as_ref() {
            f(controller.clone());
        }
    });
}

/// The controller: daemon connection state, anchor state and the
/// event receiver. Lives on the glib main loop; blocking daemon calls
/// go through the [`TangibleHost`].
pub struct Controller {
    /// Tokio runtime host for blocking daemon I/O.
    host: TangibleHost,
    /// Live daemon source (present once connected).
    source: Option<GraphQlSource>,
    /// The tracked anchor pose, or the virtual fallback.
    anchor: AnchorState,
}

impl Controller {
    /// Creates the controller and its Tokio runtime.
    pub fn new() -> Result<Rc<RefCell<Self>>, std::io::Error> {
        Ok(Rc::new(RefCell::new(Self {
            host: TangibleHost::new()?,
            source: None,
            anchor: AnchorState::default_virtual(),
        })))
    }

    /// The runtime host.
    pub fn host(&self) -> &TangibleHost {
        &self.host
    }

    /// Stores the connected source.
    pub fn set_source(&mut self, source: GraphQlSource) {
        self.source = Some(source);
    }

    /// Current anchor state.
    #[allow(dead_code)] // consumed by the map scene (P3)
    pub fn anchor(&self) -> AnchorState {
        self.anchor
    }

    /// Updates the anchor state from a tangible event.
    ///
    /// `Added`/`Moved` track the pose; `Removed`/`Lagged` fall back to
    /// the virtual anchor (the map keeps rendering while the tower is
    /// away).
    pub fn apply_event(&mut self, event: &TangibleEvent) {
        match event {
            TangibleEvent::Added(tangible) | TangibleEvent::Moved(tangible) => {
                self.anchor = AnchorState::Tracked(AnchorPose::from(tangible));
            }
            TangibleEvent::Removed(_) | TangibleEvent::Lagged(_) => {
                if let AnchorState::Tracked(pose) = self.anchor {
                    self.anchor = AnchorState::Virtual(pose);
                }
            }
        }
    }

    /// Releases all daemon registrations (`unregister_owner`).
    pub fn release_registrations(&mut self) {
        if let Some(source) = self.source.as_mut()
            && let Err(err) = self.host.unregister(source)
        {
            warn!("unregister failed: {err}");
        }
        self.source = None;
    }
}

/// Builds the controller and installs it as active.
pub fn init_controller() -> Result<Rc<RefCell<Controller>>, std::io::Error> {
    let controller = Controller::new()?;
    set_active_controller(controller.clone());
    Ok(controller)
}
