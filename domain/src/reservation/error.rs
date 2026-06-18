#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Validation failures returned by reservation domain constructors.
pub enum Error {
    #[error("minimum age must be at least one week")]
    /// Minimum age was below the accepted service-policy floor.
    EmptyMinimumAge,
    #[error("add-on label must not be empty")]
    /// Add-on label was blank after trimming.
    EmptyAddOnLabel,
    #[error("add-on label must be 120 characters or fewer")]
    /// Add-on label exceeded the customer-facing display limit.
    AddOnLabelTooLong,
}

/// Result type returned by fallible reservation error operations.
pub type Result<T> = std::result::Result<T, Error>;
