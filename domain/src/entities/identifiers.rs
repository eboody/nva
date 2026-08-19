//! Stable, validated identifiers shared by normalized entity aggregates.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

non_nil_uuid_id!(
    LocationId,
    "Stable non-nil identifier for a resort location across source imports, policies, reports, and workflows."
);
non_nil_uuid_id!(
    CustomerId,
    "Stable non-nil identifier for the customer/account responsible for pets, reservations, messages, and payments."
);
non_nil_uuid_id!(
    PetId,
    "Stable non-nil identifier for a pet whose care, temperament, vaccine, and reservation facts drive safety decisions."
);

impl std::fmt::Display for PetId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

non_nil_uuid_id!(
    DocumentId,
    "Stable non-nil identifier for a document artifact used as vaccine, waiver, medical, or incident evidence."
);
non_nil_uuid_id!(
    VaccineRecordId,
    "Stable non-nil identifier for a vaccine compliance record tied to a pet and proof document."
);

non_nil_uuid_id!(
    IncidentId,
    "Stable non-nil identifier for a pet, customer, or operational incident requiring evidence and follow-up."
);
non_nil_uuid_id!(
    MessageId,
    "Stable non-nil identifier for a customer or internal message workflow."
);
