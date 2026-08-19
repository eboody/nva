//! Audit records and identifiers for traceable pet-resort automation events.
//!
//! Audit records provide the evidence trail for agent drafts, policy decisions, manager approvals,
//! customer-message handling, and source-data repairs. They are domain facts, not log formatting.

use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::{entities, workflow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Stable non-nil identifier for an auditable operational event.
///
/// Use this id to correlate the source fact, workflow decision, review gate, and resulting staff or
/// customer action without inventing undocumented authority after the fact.
pub struct EventId(Uuid);

impl EventId {
    /// Validates an audit identity at an untrusted boundary.
    pub const fn try_new(value: Uuid) -> Result<Self, entities::NilIdentityError> {
        if value.is_nil() {
            Err(entities::NilIdentityError)
        } else {
            Ok(Self(value))
        }
    }
}

impl<'de> Deserialize<'de> for EventId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_new(Uuid::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Audit event capturing actor, subject, action, timestamp, and metadata evidence.
pub struct Event {
    /// Time the auditable action occurred.
    pub at: DateTime<Utc>,
    /// Actor accountable for the action.
    pub actor: entities::ActorRef,
    /// Domain subject affected by the action.
    pub subject: Subject,
    /// Semantic action that occurred.
    pub action: Action,
    /// Validated extension metadata retained as evidence.
    pub metadata: BTreeMap<MetadataKey, MetadataValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Subject affected by an auditable domain or workflow action.
pub enum Subject {
    /// Customer profile affected by the event.
    Customer(entities::CustomerId),
    /// Pet profile affected by the event.
    Pet(entities::PetId),
    /// Reservation affected by the event.
    Reservation(entities::reservation::Id),
    /// Resort location affected by the event.
    Location(entities::LocationId),
    /// Document affected by the event.
    Document(entities::DocumentId),
    /// Vaccine record affected by the event.
    VaccineRecord(entities::VaccineRecordId),
    /// Care note affected by the event.
    CareNote(entities::care_note::Id),
    /// Incident affected by the event.
    Incident(entities::IncidentId),
    /// Message affected by the event.
    Message(entities::MessageId),
    /// Approval record affected by the event.
    Approval(entities::approval::Id),
    /// Workflow event affected by the audit event.
    WorkflowEvent(workflow::EventId),
    /// External provider object retained as source evidence.
    External {
        /// Provider that owns the external object.
        provider: workflow::external::Provider,
        /// Provider-local object identifier.
        id: workflow::external::Id,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Auditable action category produced by staff, source ingestion, policy, approval, or automation.
pub enum Action {
    /// Customer profile facts changed.
    CustomerProfileUpdated,
    /// Pet profile facts changed.
    PetProfileUpdated,
    /// Reservation status change was suggested but not executed.
    ReservationStatusSuggested,
    /// Reservation status changed with appropriate authority.
    ReservationStatusChanged,
    /// Policy decision was recorded.
    PolicyDecisionRecorded,
    /// Document entered the intake pipeline.
    DocumentReceived,
    /// Vaccine-record review was requested.
    VaccineRecordReviewRequested,
    /// Incident lifecycle status changed.
    IncidentStatusChanged,
    /// Message approval was requested.
    MessageApprovalRequested,
    /// Approval decision was recorded.
    ApprovalDecisionRecorded,
    /// Workflow event was recorded.
    WorkflowEventRecorded,
    /// Validated extension point for domain-owned actions not yet represented by a dedicated variant.
    Extension(ActionLabel),
}

/// Validated audit action label for domain-owned extensions.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct ActionLabel(String);

/// Validated audit metadata key.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct MetadataKey(String);

/// Validated audit metadata value.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 500),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
pub struct MetadataValue(String);
