//! Staff-visible, customer-visible, and internal care-note records.

use bon::Builder;
use chrono::{DateTime, Utc};

use super::{
    actor::ActorRef,
    identifiers::{IncidentId, PetId},
    reservation,
};

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

non_nil_uuid_id!(
    Id,
    "Stable non-nil provider or source reservation identifier retained as a join key."
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Subject that a care, document, incident, audit, or message record is about.
pub enum Subject {
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Care-note category used to route safety, feeding, medication, behavior, and staff handoff information.
pub enum Kind {
    /// Feeding state or source category preserved for normalized resort records.
    Feeding,
    /// Medication state or source category preserved for normalized resort records.
    Medication,
    /// Medical state or source category preserved for normalized resort records.
    Medical,
    /// Behavior state or source category preserved for normalized resort records.
    Behavior,
    /// Grooming service line or care-note category.
    Grooming,
    /// Training service line or care-note category.
    Training,
    /// General state or source category preserved for normalized resort records.
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Visibility rule that determines whether a care note may be shown to customers or only staff.
pub enum Visibility {
    /// Internal only state or source category preserved for normalized resort records.
    InternalOnly,
    /// Customer visible state or source category preserved for normalized resort records.
    CustomerVisible,
    /// Customer visible after review state or source category preserved for normalized resort records.
    CustomerVisibleAfterReview,
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 2000),
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
pub struct Body(String);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Care note with author, visibility, subject, body, source, and review-sensitive timestamps.
pub struct CareNote {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: Id,
    /// Subject retained from source records for staff review, safety gates, and workflow joins.
    pub subject: Subject,
    /// Kind retained from source records for staff review, safety gates, and workflow joins.
    pub kind: Kind,
    /// Visibility retained from source records for staff review, safety gates, and workflow joins.
    pub visibility: Visibility,
    /// Body retained from source records for staff review, safety gates, and workflow joins.
    pub body: Body,
    /// Author retained from source records for staff review, safety gates, and workflow joins.
    pub author: ActorRef,
    /// Recorded at retained from source records for staff review, safety gates, and workflow joins.
    pub recorded_at: DateTime<Utc>,
    #[builder(default)]
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl CareNote {}
