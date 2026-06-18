use domain::{entities, policy};

/// Result type returned by fallible tools error operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
/// Decision taxonomy for error in the agent tool error boundary; each value carries operational meaning for source-grounded routing and review.
pub enum Error {
    #[error("not found: {resource} {id}")]
    /// Source-derived Resource retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    NotFound {
        /// Resource carried by this variant.
        resource: Resource,
        /// Id carried by this variant.
        id: ResourceId,
    },
    #[error("policy denied: {reason}")]
    /// Source-derived Reason retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    PolicyDenied {
        /// Reason carried by this variant.
        reason: policy::denial::Reason,
    },
    #[error("external system error: {failure}")]
    /// Source-derived Failure retained for audit, reviewer explanation, or agent context; callers must not invent or mutate it.
    External {
        /// Failure carried by this variant.
        failure: ExternalFailure,
    },
}

impl Error {
    /// Builds policy denied for the agent tool error boundary contract from validated source facts while preserving review gates and draft-only side-effect boundaries.
    pub fn policy_denied(reason: policy::denial::Reason) -> Self {
        Self::PolicyDenied { reason }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Decision taxonomy for resource in the agent tool error boundary; each value carries operational meaning for source-grounded routing and review.
pub enum Resource {
    /// Represents customer in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Customer,
    /// Represents pet in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Pet,
    /// Represents reservation in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Reservation,
    /// Represents availability snapshot in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    AvailabilitySnapshot,
    /// Represents draft reservation update in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
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
/// Decision taxonomy for resource id in the agent tool error boundary; each value carries operational meaning for source-grounded routing and review.
pub enum ResourceId {
    /// Represents customer in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Customer(entities::CustomerId),
    /// Represents pet in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Pet(entities::PetId),
    /// Represents reservation in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Reservation(entities::reservation::Id),
    /// Represents snapshot in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Snapshot(super::availability::CapacitySnapshotId),
    /// Represents draft in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
    Draft(super::draft_update::draft::Id),
    /// Represents external in the tool error decision model so the app can choose the correct evidence, review, or draft path without taking live action.
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
/// Decision taxonomy for external failure in the agent tool error boundary; each value carries operational meaning for source-grounded routing and review.
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
