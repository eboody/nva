#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by payment domain constructors.
pub enum Error {
    #[error("payment reference must not be empty")]
    /// Payment reference was blank after trimming.
    EmptyReference,
    #[error("payment reference must be 160 characters or fewer")]
    /// Payment reference exceeded the accepted storage length.
    ReferenceTooLong,
    #[error("paid deposit requires payment reference")]
    /// Persisted or promoted paid deposits must include reconciliation evidence.
    PaidDepositMissingReference,
    #[error("unpaid deposit status must not carry payment reference")]
    /// Persisted or promoted unpaid deposits must not claim reconciliation evidence.
    UnpaidDepositHasReference,
}

/// Result type returned by fallible payment error operations.
pub type Result<T> = std::result::Result<T, Error>;
