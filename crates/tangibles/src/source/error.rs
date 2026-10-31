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
}
