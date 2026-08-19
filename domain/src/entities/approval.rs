//! Review-gated approval records and lifecycle evidence.

use super::{
    actor::ActorRef,
    identifiers::{DocumentId, IncidentId, MessageId, VaccineRecordId},
    reservation,
};
use crate::policy;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

non_nil_uuid_id!(
    Id,
    "Stable non-nil provider or source reservation identifier retained as a join key."
);

/// Approval aggregate construction and rehydration failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("approval id is required")]
    /// Represents the `IdRequired` semantic case.
    IdRequired,
    #[error("approval target is required")]
    /// Represents the `TargetRequired` semantic case.
    TargetRequired,
    #[error("approval review gate is required")]
    /// Represents the `GateRequired` semantic case.
    GateRequired,
    #[error("approval lifecycle is required")]
    /// Represents the `LifecycleRequired` semantic case.
    LifecycleRequired,
    #[error("approval requester is required")]
    /// Represents the `RequestedByRequired` semantic case.
    RequestedByRequired,
    #[error("approval request time is required")]
    /// Represents the `RequestedAtRequired` semantic case.
    RequestedAtRequired,
    #[error("approval decision time cannot precede request time")]
    /// Represents the `DecisionTimePrecedesRequest` semantic case.
    DecisionTimePrecedesRequest,
    #[error("approval review gate does not match approval target")]
    /// Represents the `GateTargetMismatch` semantic case.
    GateTargetMismatch,
}

/// Result alias for approval aggregate construction and rehydration.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Approval record showing who decided, what target was reviewed, and what lifecycle state resulted.
pub struct Record {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    id: Id,
    /// Target retained from source records for staff review, safety gates, and workflow joins.
    target: Target,
    /// Gate retained from source records for staff review, safety gates, and workflow joins.
    gate: policy::ReviewGate,
    /// Lifecycle retained from source records for staff review, safety gates, and workflow joins.
    lifecycle: Lifecycle,
    /// Requested by retained from source records for staff review, safety gates, and workflow joins.
    requested_by: ActorRef,
    /// Requested at retained from source records for staff review, safety gates, and workflow joins.
    requested_at: DateTime<Utc>,
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Serialize, Deserialize)]
struct RawRecord {
    id: Id,
    target: Target,
    gate: policy::ReviewGate,
    lifecycle: Lifecycle,
    requested_by: ActorRef,
    requested_at: DateTime<Utc>,
    audit_refs: Vec<crate::audit::EventId>,
}

impl RawRecord {
    fn try_into_record(self) -> Result<Record> {
        validate_gate_target(&self.gate, &self.target)?;
        if self
            .lifecycle
            .decision_actor_and_time()
            .is_some_and(|(_, decided_at)| decided_at < self.requested_at)
        {
            return Err(Error::DecisionTimePrecedesRequest);
        }
        Ok(Record {
            id: self.id,
            target: self.target,
            gate: self.gate,
            lifecycle: self.lifecycle,
            requested_by: self.requested_by,
            requested_at: self.requested_at,
            audit_refs: self.audit_refs,
        })
    }
}

impl<'de> Deserialize<'de> for Record {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawRecord::deserialize(deserializer)?
            .try_into_record()
            .map_err(serde::de::Error::custom)
    }
}

impl Record {
    /// Starts a checked approval aggregate builder.
    pub fn builder() -> RecordBuilder {
        RecordBuilder::default()
    }

    /// Approval id used by audit, storage, and authority evidence.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Review target this approval applies to.
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// Review gate this approval applies to.
    pub fn gate(&self) -> &policy::ReviewGate {
        &self.gate
    }

    /// Lifecycle state carried by this approval record.
    pub fn lifecycle(&self) -> &Lifecycle {
        &self.lifecycle
    }

    /// Actor that requested this approval.
    pub fn requested_by(&self) -> &ActorRef {
        &self.requested_by
    }

    /// Timestamp when approval was requested.
    pub fn requested_at(&self) -> DateTime<Utc> {
        self.requested_at
    }

    /// Audit refs attached to this approval.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }

    /// Reports whether this approval gate currently applies to the target workflow.
    pub fn is_applicable(&self) -> bool {
        matches!(self.lifecycle, Lifecycle::Approved { .. })
    }
}

/// Builder for checked approval aggregates.
#[derive(Debug, Clone, Default)]
pub struct RecordBuilder {
    id: Option<Id>,
    target: Option<Target>,
    gate: Option<policy::ReviewGate>,
    lifecycle: Option<Lifecycle>,
    requested_by: Option<ActorRef>,
    requested_at: Option<DateTime<Utc>>,
    audit_refs: Vec<crate::audit::EventId>,
}

impl RecordBuilder {
    /// Returns the aggregate id.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
    /// Returns the aggregate target.
    pub fn target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
    }
    /// Returns the aggregate gate.
    pub fn gate(mut self, gate: policy::ReviewGate) -> Self {
        self.gate = Some(gate);
        self
    }
    /// Returns the aggregate lifecycle.
    pub fn lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }
    /// Returns the aggregate requested by.
    pub fn requested_by(mut self, requested_by: ActorRef) -> Self {
        self.requested_by = Some(requested_by);
        self
    }
    /// Returns the aggregate requested at.
    pub fn requested_at(mut self, requested_at: DateTime<Utc>) -> Self {
        self.requested_at = Some(requested_at);
        self
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(mut self, audit_refs: Vec<crate::audit::EventId>) -> Self {
        self.audit_refs = audit_refs;
        self
    }
    /// Validates the accumulated fields and builds the aggregate.
    pub fn build(self) -> Result<Record> {
        RawRecord {
            id: self.id.ok_or(Error::IdRequired)?,
            target: self.target.ok_or(Error::TargetRequired)?,
            gate: self.gate.ok_or(Error::GateRequired)?,
            lifecycle: self.lifecycle.ok_or(Error::LifecycleRequired)?,
            requested_by: self.requested_by.ok_or(Error::RequestedByRequired)?,
            requested_at: self.requested_at.ok_or(Error::RequestedAtRequired)?,
            audit_refs: self.audit_refs,
        }
        .try_into_record()
    }
}

fn validate_gate_target(gate: &policy::ReviewGate, target: &Target) -> Result<()> {
    let legal = matches!(
        (gate, target),
        (
            policy::ReviewGate::CustomerMessageApproval,
            Target::Message(_)
        ) | (
            policy::ReviewGate::MedicalDocumentReview,
            Target::Document(_)
        ) | (
            policy::ReviewGate::MedicalDocumentReview,
            Target::VaccineRecord(_)
        ) | (policy::ReviewGate::ManagerApproval, Target::Reservation(_))
            | (policy::ReviewGate::ManagerApproval, Target::Incident(_))
            | (
                policy::ReviewGate::RefundOrDepositException,
                Target::Reservation(_)
            )
            | (policy::ReviewGate::BehaviorReview, Target::Reservation(_))
            | (policy::ReviewGate::BehaviorReview, Target::Incident(_))
    );
    if legal {
        Ok(())
    } else {
        Err(Error::GateTargetMismatch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Operational artifact that an approval gate is allowed to approve, reject, or mark non-applicable.
pub enum Target {
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Customer or pet document participating in review.
    Document(DocumentId),
    /// Vaccination document or status record under review.
    VaccineRecord(VaccineRecordId),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
    /// Customer communication record participating in approval.
    Message(MessageId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Approval lifecycle state for draft, requested, approved, rejected, or non-applicable review gates.
pub enum Lifecycle {
    /// Approval requested state or source category preserved for normalized resort records.
    ApprovalRequested,
    /// Approved state or source category preserved for normalized resort records.
    Approved {
        /// Decided by retained from source records for staff review, safety gates, and workflow joins.
        decided_by: ActorRef,
        /// Decided at retained from source records for staff review, safety gates, and workflow joins.
        decided_at: DateTime<Utc>,
    },
    /// Rejected state or source category preserved for normalized resort records.
    Rejected {
        /// Decided by retained from source records for staff review, safety gates, and workflow joins.
        decided_by: ActorRef,
        /// Decided at retained from source records for staff review, safety gates, and workflow joins.
        decided_at: DateTime<Utc>,
    },
    /// Reservation is no longer active.
    Cancelled,
    /// Superseded state or source category preserved for normalized resort records.
    Superseded,
}

impl Lifecycle {
    /// Returns the accountable actor and timestamp when the review reached a terminal decision.
    pub fn decision_actor_and_time(&self) -> Option<(&ActorRef, DateTime<Utc>)> {
        match self {
            Self::Approved {
                decided_by,
                decided_at,
            }
            | Self::Rejected {
                decided_by,
                decided_at,
            } => Some((decided_by, *decided_at)),
            Self::ApprovalRequested | Self::Cancelled | Self::Superseded => None,
        }
    }
}
