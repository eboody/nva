use domain::{entities, policy};

/// Result type returned by fallible tools error operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
/// Decision choices for error in the agent tool error surface; each value guides source-grounded routing and review.
pub enum Error {
    #[error("policy denied: {reason}")]
    /// Reason copied from reviewed source input for audit, reviewer explanation,
}

impl Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Decision choices for resource in the agent tool error surface; each value guides source-grounded routing and review.
pub enum Resource {
    /// Selects customer for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Customer,
    /// Selects pet for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Pet,
    /// Selects reservation for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Reservation,
    /// Selects availability snapshot for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    AvailabilitySnapshot,
    /// Selects draft reservation update for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
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
/// Decision choices for resource id in the agent tool error surface; each value guides source-grounded routing and review.
pub enum ResourceId {
    /// Selects customer for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Customer(entities::CustomerId),
    /// Selects pet for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Pet(entities::PetId),
    /// Selects reservation for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Reservation(entities::reservation::Id),
    /// Selects snapshot for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Snapshot(super::availability::CapacitySnapshotId),
    /// Selects draft for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    Draft(super::draft_update::draft::Id),
    /// Selects external for the tool error decision model so the app can choose a review, evidence, or draft path without taking live action.
    External(String),
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Customer(id) => write!(formatter, "{}", id.get()),
            Self::Pet(id) => write!(formatter, "{}", id.get()),
            Self::Reservation(id) => write!(formatter, "{}", id.get()),
            Self::Snapshot(id) => formatter.write_str(id.clone().into_inner().as_str()),
            Self::Draft(id) => formatter.write_str(id.clone().into_inner().as_str()),
            Self::External(id) => formatter.write_str(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Decision choices for external failure in the agent tool error surface; each value guides source-grounded routing and review.
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

#[cfg(test)]
mod changed_line_tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn entity_resource_ids_render_their_semantic_identifiers() {
        let customer = entities::CustomerId::new(Uuid::from_u128(1));
        let pet = entities::PetId::new(Uuid::from_u128(2));
        let reservation = entities::reservation::Id::new(Uuid::from_u128(3));

        assert_eq!(
            ResourceId::Customer(customer).to_string(),
            customer.get().to_string()
        );
        assert_eq!(ResourceId::Pet(pet).to_string(), pet.get().to_string());
        assert_eq!(
            ResourceId::Reservation(reservation).to_string(),
            reservation.get().to_string()
        );
    }
}
