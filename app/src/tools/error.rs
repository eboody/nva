use thiserror::Error;

use domain::policy;

pub type Result<T> = std::result::Result<T, ToolError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ToolError {
    #[error("not found: {resource} {id}")]
    NotFound {
        resource: ToolResource,
        id: ToolResourceId,
    },
    #[error("policy denied: {reason}")]
    PolicyDenied { reason: policy::denial::Reason },
    #[error("external system error: {failure}")]
    External { failure: ExternalFailure },
}

impl ToolError {
    pub fn policy_denied(reason: policy::denial::Reason) -> Self {
        Self::PolicyDenied { reason }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolResource {
    Customer,
    Pet,
    Reservation,
    AvailabilitySnapshot,
    DraftReservationUpdate,
}

impl std::fmt::Display for ToolResource {
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
pub enum ToolResourceId {
    Customer(domain::entities::CustomerId),
    Pet(domain::entities::PetId),
    Reservation(domain::entities::ReservationId),
    Snapshot(super::CapacitySnapshotId),
    Draft(super::DraftId),
    External(String),
}

impl std::fmt::Display for ToolResourceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Customer(id) => write!(formatter, "{}", id.0),
            Self::Pet(id) => write!(formatter, "{}", id.0),
            Self::Reservation(id) => write!(formatter, "{}", id.0),
            Self::Snapshot(id) => formatter.write_str(id.clone().into_inner().as_str()),
            Self::Draft(id) => formatter.write_str(id.0.clone().into_inner().as_str()),
            Self::External(id) => formatter.write_str(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalFailure {
    PortalUnavailable,
    PaymentProviderUnavailable,
    MessageProviderUnavailable,
    StorageUnavailable,
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
