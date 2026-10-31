use async_channel::Receiver;
use qdo36_tangibles::AnchorPattern;
use qdo36_tangibles::DaemonError;
use qdo36_tangibles::GraphQlSource;
use qdo36_tangibles::PatternId;
use qdo36_tangibles::SurfaceMm;
use qdo36_tangibles::TangibleEvent;
use qdo36_tangibles::TangibleSource;

/// Owns the Tokio runtime the daemon client runs on.
///
/// GTK runs the glib main loop on the main thread; the websocket
/// subscription task of `GraphQlSource` needs a Tokio reactor, hosted
/// here on its own runtime (multi-thread scheduler drives spawned
/// tasks without `block_on` staying parked).
pub struct TangibleHost {
    /// The Tokio runtime driving daemon I/O.
    runtime: tokio::runtime::Runtime,
}

impl TangibleHost {
    /// Creates the runtime.
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            runtime: tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build()?,
        })
    }

    /// A `Send` handle to the runtime - lets worker threads drive
    /// `block_on` without touching the non-`Send` controller.
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }

    /// Full daemon handshake on the runtime: connect, register the
    /// anchor profile, open the event stream - in one `block_on` so
    /// the websocket subscription task stays scheduled on this
    /// runtime. Callable from any thread via [`Self::handle`].
    pub fn connect_and_subscribe(
        handle: &tokio::runtime::Handle,
        endpoint: &str,
        pattern: &AnchorPattern,
    ) -> Result<(GraphQlSource, PatternId, Receiver<TangibleEvent>), DaemonError> {
        handle.block_on(async {
            let mut source = GraphQlSource::connect(endpoint).await?;
            let pattern_id = source.register_anchor_profile(pattern).await?;
            let events = source.tangible_events(pattern_id).await;
            Ok((source, pattern_id, events))
        })
    }

    /// Queries the surface size in millimeters.
    pub fn surface(&self, source: &GraphQlSource) -> Result<SurfaceMm, DaemonError> {
        self.runtime.block_on(source.surface())
    }

    /// Removes this app's registrations on the daemon - the anchor
    /// pattern and every resource it owns.
    pub fn unregister(&self, source: &mut GraphQlSource) -> Result<(), DaemonError> {
        self.runtime.block_on(source.unregister())
    }
}
