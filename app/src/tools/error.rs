use domain::{entities, policy};

/// Result type returned by fallible tools error operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
/// Classifies error values that drive the agent tool error boundary.
pub enum Error {
    #[error("not found: {resource} {id}")]
    /// Resource preserved as evidence for audit, review, or agent context.
    NotFound {
        /// Resource carried by this variant.
        resource: Resource,
        /// Id carried by this variant.
        id: ResourceId,
    },
    #[error("policy denied: {reason}")]
    /// Reason preserved as evidence for audit, review, or agent context.
    PolicyDenied {
        /// Reason carried by this variant.
        reason: policy::denial::Reason,
    },
    #[error("external system error: {failure}")]
    /// Failure preserved as evidence for audit, review, or agent context.
    External {
        /// Failure carried by this variant.
        failure: ExternalFailure,
    },
}

impl Error {
    /// Builds or derives policy denied data for the agent tool error boundary contract.
    pub fn policy_denied(reason: policy::denial::Reason) -> Self {
        Self::PolicyDenied { reason }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Classifies resource values that drive the agent tool error boundary.
pub enum Resource {
    /// Routes tool error work flagged as customer to the right queue, review gate, or agent packet.
    Customer,
    /// Routes tool error work flagged as pet to the right queue, review gate, or agent packet.
    Pet,
    /// Routes tool error work flagged as reservation to the right queue, review gate, or agent packet.
    Reservation,
    /// Routes tool error work flagged as availability snapshot to the right queue, review gate, or agent packet.
    AvailabilitySnapshot,
    /// Routes tool error work flagged as draft reservation update to the right queue, review gate, or agent packet.
    DraftReservationUpdate,
}

impl std::fmt::Display for Resource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Customer => "customer",
            Self::Pet => "pet",
            Self::Reservation => "reservation",
            Self::AvailabilitySnapshot => "availability snapshot",
            Self::DraftReservationUpdate => "draft reservation update",
        };
        formatter.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Classifies resource id values that drive the agent tool error boundary.
pub enum ResourceId {
    /// Routes tool error work flagged as customer to the right queue, review gate, or agent packet.
    Customer(entities::CustomerId),
    /// Routes tool error work flagged as pet to the right queue, review gate, or agent packet.
    Pet(entities::PetId),
    /// Routes tool error work flagged as reservation to the right queue, review gate, or agent packet.
    Reservation(entities::reservation::Id),
    /// Routes tool error work flagged as snapshot to the right queue, review gate, or agent packet.
    Snapshot(super::availability::CapacitySnapshotId),
    /// Routes tool error work flagged as draft to the right queue, review gate, or agent packet.
    Draft(super::draft_update::draft::Id),
    /// Routes tool error work flagged as external to the right queue, review gate, or agent packet.
    External(String),
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Customer(id) => write!(formatter, "{}", id.0),
            Self::Pet(id) => write!(formatter, "{}", id.0),
            Self::Reservation(id) => write!(formatter, "{}", id.0),
            Self::Snapshot(id) => formatter.write_str(id.clone().into_inner().as_str()),
            Self::Draft(id) => formatter.write_str(id.clone().into_inner().as_str()),
            Self::External(id) => formatter.write_str(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Classifies external failure values that drive the agent tool error boundary.
pub enum ExternalFailure {
    /// Identifies portal unavailable as the reason the workflow must stop, retry, or request review.
    PortalUnavailable,
    /// Identifies payment provider unavailable as the reason the workflow must stop, retry, or request review.
    PaymentProviderUnavailable,
    /// Identifies message provider unavailable as the reason the workflow must stop, retry, or request review.
    MessageProviderUnavailable,
    /// Identifies storage unavailable as the reason the workflow must stop, retry, or request review.
    StorageUnavailable,
    /// Identifies other as the reason the workflow must stop, retry, or request review.
    Other(String),
}

impl std::fmt::Display for ExternalFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::PortalUnavailable => "portal unavailable",
            Self::PaymentProviderUnavailable => "payment provider unavailable",
            Self::MessageProviderUnavailable => "message provider unavailable",
            Self::StorageUnavailable => "storage unavailable",
            Self::Other(message) => return formatter.write_str(message),
        };
        formatter.write_str(label)
    }
}
