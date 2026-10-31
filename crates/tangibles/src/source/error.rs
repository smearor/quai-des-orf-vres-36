use thiserror::Error;

/// Errors talking to the `tangible-objects` daemon.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// The GraphQL endpoint is unreachable or refused the connection.
    #[error("daemon unreachable at {url}: {reason}")]
    Connect {
        /// The configured daemon URL.
        url: String,
        /// The underlying error message.
        reason: String,
    },
    /// A GraphQL query/subscription failed.
    #[error("daemon query failed: {0}")]
    Query(String),
    /// The event subscription stream ended or errored.
    #[error("subscription failed: {0}")]
    Subscription(String),
    /// A bundled figure profile is malformed.
    #[error("invalid profile: {0}")]
    Profile(String),
    /// Pattern or field-set registration failed.
    #[error("registration failed: {0}")]
    Registration(String),
    /// No answer within the expected time.
    #[error("daemon timed out: {0}")]
    Timeout(String),
    /// The HTTP/WebSocket transport failed.
    #[error("daemon transport failed: {0}")]
    Transport(String),
    /// The endpoint returned a GraphQL `errors` array.
    #[error("GraphQL error: {0}")]
    GraphQl(String),
    /// A response could not be decoded into the expected type.
    #[error("response decode failed: {0}")]
    Decode(String),
    /// The daemon cannot serve the request (missing device info,
    /// pattern, ...).
    #[error("daemon unavailable: {0}")]
    Unavailable(String),
}

impl From<tangible_objects_client::ClientError> for DaemonError {
    fn from(err: tangible_objects_client::ClientError) -> Self {
        use tangible_objects_client::ClientError as C;
        match err {
            C::Http(_) | C::Ws(_) | C::NoData => Self::Transport(err.to_string()),
            C::Subscription(_) | C::Protocol(_) | C::FieldNotFound(_) => Self::Subscription(err.to_string()),
            C::Graphql(value) => Self::GraphQl(value.to_string()),
            C::Decode(err) => Self::Decode(err.to_string()),
        }
    }
}
