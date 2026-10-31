use crate::AnchorPattern;
use crate::DaemonError;
use crate::SurfaceMm;
use crate::TangibleEvent;
use crate::TangibleSource;
use crate::source::REGISTRATION_OWNER;
use async_channel::Receiver;
use futures_lite::StreamExt;
use tangible_objects_client::Endpoint;
use tangible_objects_client::GraphQLClient;
use tangible_objects_client::ProfileQuery;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::RawAxisValue;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use tangible_objects_model::UnitsPerMm;
use tracing::debug;
use tracing::warn;

/// Tangible source backed by the `tangible-objects` daemon.
///
/// Thin adapter over the upstream [`GraphQLClient`] (HTTP queries and
/// mutations, `graphql-transport-ws` subscriptions): the typed stream
/// of upstream `TangibleEvent`s is projected into our narrower
/// [`TangibleEvent`] channel so the rest of the crate never sees
/// non-tangible payload.
///
/// `connect` expects a Tokio runtime on the calling thread - the GTK
/// app hosts a dedicated runtime thread for it.
pub struct GraphQlSource {
    /// The upstream typed client.
    client: GraphQLClient,
    /// Our registration owner for session-lifetime cleanup.
    owner: Owner,
}

impl GraphQlSource {
    /// Connects to the daemon at `endpoint` (e.g.
    /// `http://127.0.0.1:8080` or the full `…/graphql` URL).
    pub async fn connect(endpoint: &str) -> Result<Self, DaemonError> {
        let url = if endpoint.ends_with("/graphql") {
            endpoint.to_string()
        } else {
            format!("{}/graphql", endpoint.trim_end_matches('/'))
        };
        let client = GraphQLClient::new(Endpoint::new(url.clone())).map_err(|e| DaemonError::Connect { url, reason: e.to_string() })?;
        Ok(Self {
            client,
            owner: Owner::new(REGISTRATION_OWNER),
        })
    }
}

impl TangibleSource for GraphQlSource {
    async fn register_anchor_profile(&mut self, profile: &AnchorPattern) -> Result<PatternId, DaemonError> {
        self.client.register_pattern(profile.profile(), Some(&self.owner), Lifetime::Session).await?;
        // registerPattern returns bool - resolve the assigned runtime
        // pattern id via the profiles query filtered to our owner.
        let infos = self
            .client
            .profiles(&ProfileQuery {
                owner: Some(self.owner.clone()),
                static_only: Some(false),
            })
            .await?;
        infos
            .iter()
            .find(|info| info.name == profile.profile().name)
            .map(|info| info.pattern_id)
            .ok_or_else(|| DaemonError::Unavailable(format!("registered pattern '{}' not found", profile.profile().name)))
    }

    async fn tangible_events(&self, pattern: PatternId) -> Receiver<TangibleEvent> {
        let (tx, receiver) = async_channel::bounded(256);
        match self.client.tangible_events(Some(pattern)).await {
            Ok(stream) => {
                let mut stream = Box::pin(stream);
                tokio::spawn(async move {
                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(event) => {
                                if let Some(projected) = TangibleEvent::project(&event)
                                    && tx.try_send(projected).is_err()
                                {
                                    warn!("tangible event channel full/closed - dropping event");
                                }
                            }
                            Err(err) => {
                                warn!("tangible event stream error: {err} - asking caller to resync");
                                let _ = tx.try_send(TangibleEvent::lagged(None));
                            }
                        }
                    }
                    debug!("tangible event subscription ended");
                });
            }
            Err(err) => {
                warn!("tangibleEvents subscription failed: {err} - channel carries Lagged only");
                let _ = tx.try_send(TangibleEvent::lagged(None));
            }
        }
        receiver
    }

    async fn tangibles(&self) -> Result<Vec<TrackedTangible>, DaemonError> {
        Ok(self.client.tangibles().await?)
    }

    async fn touch_points(&self) -> Result<Vec<TouchPoint>, DaemonError> {
        Ok(self.client.touch_points().await?)
    }

    async fn surface(&self) -> Result<SurfaceMm, DaemonError> {
        let info = self
            .client
            .device_info()
            .await?
            .ok_or_else(|| DaemonError::Unavailable("daemon reports no device info".to_string()))?;
        let mm = |res: Option<UnitsPerMm>, min: Option<RawAxisValue>, max: Option<RawAxisValue>| -> Option<Millimeters> {
            let res = res?.as_f64();
            let span = max?.as_f64() - min?.as_f64();
            (res > 0.0).then(|| Millimeters::new(span / res))
        };
        let width = mm(info.x_res, info.x_min, info.x_max);
        let height = mm(info.y_res, info.y_min, info.y_max);
        width
            .zip(height)
            .map(SurfaceMm::from)
            .ok_or_else(|| DaemonError::Unavailable("device info lacks resolution/extent".to_string()))
    }

    async fn unregister(&mut self) -> Result<(), DaemonError> {
        self.client.unregister_owner(&self.owner).await?;
        Ok(())
    }
}
