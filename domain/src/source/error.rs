#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by source domain constructors.
pub enum Error {
    #[error("timestamp must not be empty")]
    /// Signals that timestamp was blank or missing during source validation.
    EmptyTimestamp,
    #[error("timestamp must be RFC3339 UTC-compatible text")]
    /// Signals that timestamp could not be parsed or accepted during source validation.
    InvalidTimestamp,
    #[error("source endpoint must not be empty")]
    /// Signals that endpoint was blank or missing during source validation.
    EmptyEndpoint,

    #[error("extraction batch id must not be empty")]
    /// Signals that extraction batch was blank or missing during source validation.
    EmptyExtractionBatch,
    #[error("request scope must not be empty")]
    /// Signals that request scope was blank or missing during source validation.
    EmptyRequestScope,
    #[error("schema version must not be empty")]
    /// Signals that schema version was blank or missing during source validation.
    EmptySchemaVersion,

    #[error("source payload hash must not be empty")]
    /// Signals that payload hash was blank or missing during source validation.
    EmptyPayloadHash,
    #[error("raw payload reference must not be empty")]
    /// Signals that raw payload ref was blank or missing during source validation.
    EmptyRawPayloadRef,
    #[error("observed status must not be empty")]
    /// Signals that observed status was blank or missing during source validation.
    EmptyObservedStatus,
}

/// Result type returned by fallible source operations.
pub type Result<T> = std::result::Result<T, Error>;
