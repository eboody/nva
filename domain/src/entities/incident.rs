//! Incident aggregates and target-bound closure authority.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use super::{
    actor::ActorRef,
    approval,
    identifiers::{CustomerId, IncidentId, LocationId, PetId},
    reservation,
};
use crate::{incident, policy};

/// Checked evidence used to close an incident lifecycle.
pub mod incident_record {
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    use super::{ActorRef, IncidentError, IncidentId, approval, policy};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Serializable historical proof that an exact incident received manager approval for closure.
    pub struct ClosureEvidence {
        approval_id: approval::Id,
        incident_id: IncidentId,
        decided_by: ActorRef,
        decided_at: DateTime<Utc>,
    }

    /// Opaque one-use permission to close one exact incident.
    pub struct IncidentClosureAuthority {
        pub(super) evidence: ClosureEvidence,
    }

    impl ClosureEvidence {
        /// Promotes an exact approved incident decision into closure evidence.
        pub fn try_from_approval(
            approval: &approval::Record,
            incident_id: IncidentId,
        ) -> Result<Self, IncidentError> {
            if approval.target() != &approval::Target::Incident(incident_id)
                || approval.gate() != &policy::ReviewGate::ManagerApproval
            {
                return Err(IncidentError::ClosureApprovalMismatch);
            }
            let approval::Lifecycle::Approved {
                decided_by,
                decided_at,
            } = approval.lifecycle()
            else {
                return Err(IncidentError::ClosureApprovalMismatch);
            };
            Ok(Self {
                approval_id: approval.id(),
                incident_id,
                decided_by: decided_by.clone(),
                decided_at: *decided_at,
            })
        }

        /// Exact incident target approved for closure.
        pub const fn incident_id(&self) -> IncidentId {
            self.incident_id
        }

        /// Reviewer identity recorded by the accepted historical decision.
        pub const fn decided_by(&self) -> &ActorRef {
            &self.decided_by
        }
    }

    /// Test-only issuer for the incident closure capability protocol.
    ///
    /// Production issuance is deliberately absent until a real authenticated reviewer boundary
    /// owns current actor, role, and scope validation.
    #[cfg(test)]
    pub(crate) fn issue_incident_closure_authority(
        approval: &approval::Record,
        incident_id: IncidentId,
        current_reviewer: &ActorRef,
    ) -> Result<IncidentClosureAuthority, IncidentError> {
        let evidence = ClosureEvidence::try_from_approval(approval, incident_id)?;
        if evidence.decided_by() != current_reviewer {
            return Err(IncidentError::ClosureApprovalMismatch);
        }
        Ok(IncidentClosureAuthority { evidence })
    }

    #[cfg(test)]
    mod tests {
        use chrono::{TimeZone, Utc};
        use uuid::Uuid;

        use super::{super::Incident, *};
        use crate::{entities, incident, policy};

        #[test]
        fn current_reviewer_authority_issues_one_incident_bound_closure_capability() {
            let incident_id = IncidentId::new(uuid::Uuid::from_u128(1));
            let reviewer = ActorRef::Manager {
                manager_id: entities::ManagerId::try_new("manager-1").unwrap(),
            };
            let decided_at = Utc.with_ymd_and_hms(2026, 8, 15, 1, 0, 0).unwrap();
            let approval = approval::Record::builder()
                .id(approval::Id::new(Uuid::from_u128(2)))
                .target(approval::Target::Incident(incident_id))
                .gate(policy::ReviewGate::ManagerApproval)
                .lifecycle(approval::Lifecycle::Approved {
                    decided_by: reviewer.clone(),
                    decided_at,
                })
                .requested_by(ActorRef::System)
                .requested_at(decided_at)
                .build()
                .unwrap();
            let incident = Incident::builder()
                .id(incident_id)
                .location_id(entities::LocationId::new(Uuid::from_u128(3)))
                .primary_subject(entities::IncidentSubject::Pet(entities::PetId::new(
                    Uuid::from_u128(4),
                )))
                .category(incident::Category::Medication)
                .severity(incident::Severity::High)
                .status(incident::Status::NeedsManagerReview)
                .reported_by(ActorRef::System)
                .reported_at(decided_at)
                .summary(incident::Summary::try_new("missed dose").unwrap())
                .required_review_gates(vec![policy::ReviewGate::ManagerApproval])
                .build()
                .unwrap();

            let authority =
                issue_incident_closure_authority(&approval, incident_id, &reviewer).unwrap();
            let closed = incident.close_with(authority).unwrap();

            assert_eq!(closed.status(), incident::Status::Closed);
            assert!(closed.closure_evidence().is_some());
        }
    }
}

/// Incident aggregate construction and rehydration failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IncidentError {
    #[error("incident id is required")]
    /// Represents the `IdRequired` semantic case.
    IdRequired,
    #[error("incident location id is required")]
    /// Represents the `LocationIdRequired` semantic case.
    LocationIdRequired,
    #[error("incident primary subject is required")]
    /// Represents the `PrimarySubjectRequired` semantic case.
    PrimarySubjectRequired,
    #[error("incident category is required")]
    /// Represents the `CategoryRequired` semantic case.
    CategoryRequired,
    #[error("incident severity is required")]
    /// Represents the `SeverityRequired` semantic case.
    SeverityRequired,
    #[error("incident status is required")]
    /// Represents the `StatusRequired` semantic case.
    StatusRequired,
    #[error("incident reporter is required")]
    /// Represents the `ReportedByRequired` semantic case.
    ReportedByRequired,
    #[error("incident report time is required")]
    /// Represents the `ReportedAtRequired` semantic case.
    ReportedAtRequired,
    #[error("incident summary is required")]
    /// Represents the `SummaryRequired` semantic case.
    SummaryRequired,
    #[error("incident requires manager approval review gate")]
    /// Represents the `IncidentRequiresManagerApprovalReviewGate` semantic case.
    IncidentRequiresManagerApprovalReviewGate,
    #[error("customer-message incident requires customer message approval review gate")]
    /// Represents the `CustomerMessageIncidentRequiresCustomerMessageApprovalReviewGate` semantic case.
    CustomerMessageIncidentRequiresCustomerMessageApprovalReviewGate,
    #[error("incident closure evidence is bound to a different target or decision")]
    /// Closure approval was not an approved manager decision for this incident.
    ClosureApprovalMismatch,
    #[error("only a closed incident may carry closure evidence")]
    /// Historical closure evidence cannot be attached to an active incident state.
    ClosureEvidenceRequiresClosedStatus,
    #[error("closed incident rehydration requires a trusted approval aggregate")]
    /// Generic serde cannot promote historical closure fields into a closed lifecycle.
    ClosedRehydrationRequiresTrustedApproval,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
/// Incident record used for manager attention, safety follow-up, customer messaging, and audit evidence.
pub struct Incident {
    id: IncidentId,
    location_id: LocationId,
    primary_subject: IncidentSubject,
    category: incident::Category,
    severity: incident::Severity,
    status: incident::Status,
    reported_by: ActorRef,
    reported_at: DateTime<Utc>,
    summary: incident::Summary,
    required_review_gates: Vec<policy::ReviewGate>,
    closure_evidence: Option<incident_record::ClosureEvidence>,
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawIncident {
    id: IncidentId,
    location_id: LocationId,
    primary_subject: IncidentSubject,
    category: incident::Category,
    severity: incident::Severity,
    status: incident::Status,
    reported_by: ActorRef,
    reported_at: DateTime<Utc>,
    summary: incident::Summary,
    required_review_gates: Vec<policy::ReviewGate>,
    closure_evidence: Option<serde::de::IgnoredAny>,
    audit_refs: Vec<crate::audit::EventId>,
}

impl RawIncident {
    fn try_into_incident(self) -> std::result::Result<Incident, IncidentError> {
        if matches!(
            self.severity,
            incident::Severity::High | incident::Severity::Critical
        ) && !self
            .required_review_gates
            .contains(&policy::ReviewGate::ManagerApproval)
        {
            return Err(IncidentError::IncidentRequiresManagerApprovalReviewGate);
        }
        if matches!(self.status, incident::Status::CustomerMessageReview)
            && !self
                .required_review_gates
                .contains(&policy::ReviewGate::CustomerMessageApproval)
        {
            return Err(
                IncidentError::CustomerMessageIncidentRequiresCustomerMessageApprovalReviewGate,
            );
        }
        if matches!(self.status, incident::Status::Closed) {
            return Err(IncidentError::ClosedRehydrationRequiresTrustedApproval);
        } else if self.closure_evidence.is_some() {
            return Err(IncidentError::ClosureEvidenceRequiresClosedStatus);
        }
        Ok(Incident {
            id: self.id,
            location_id: self.location_id,
            primary_subject: self.primary_subject,
            category: self.category,
            severity: self.severity,
            status: self.status,
            reported_by: self.reported_by,
            reported_at: self.reported_at,
            summary: self.summary,
            required_review_gates: self.required_review_gates,
            closure_evidence: None,
            audit_refs: self.audit_refs,
        })
    }
}

impl<'de> Deserialize<'de> for Incident {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawIncident::deserialize(deserializer)?
            .try_into_incident()
            .map_err(serde::de::Error::custom)
    }
}

impl Incident {
    /// Starts checked construction of the aggregate.
    pub fn builder() -> IncidentBuilder {
        IncidentBuilder::default()
    }
    /// Closes this incident only by consuming opaque current authority for this exact target.
    pub fn close_with(
        mut self,
        authority: incident_record::IncidentClosureAuthority,
    ) -> std::result::Result<Self, IncidentError> {
        let evidence = authority.evidence;
        if evidence.incident_id() != self.id {
            return Err(IncidentError::ClosureApprovalMismatch);
        }
        self.status = incident::Status::Closed;
        self.closure_evidence = Some(evidence);
        Ok(self)
    }

    /// Returns persisted historical closure evidence, when the incident is closed.
    pub const fn closure_evidence(&self) -> Option<&incident_record::ClosureEvidence> {
        self.closure_evidence.as_ref()
    }

    /// Returns the aggregate id.
    pub fn id(&self) -> IncidentId {
        self.id
    }
    /// Returns the aggregate location id.
    pub fn location_id(&self) -> LocationId {
        self.location_id
    }
    /// Returns the aggregate primary subject.
    pub fn primary_subject(&self) -> &IncidentSubject {
        &self.primary_subject
    }
    /// Returns the aggregate category.
    pub fn category(&self) -> incident::Category {
        self.category
    }
    /// Returns the aggregate severity.
    pub fn severity(&self) -> incident::Severity {
        self.severity
    }
    /// Returns the aggregate status.
    pub fn status(&self) -> incident::Status {
        self.status
    }
    /// Returns the aggregate reported by.
    pub fn reported_by(&self) -> &ActorRef {
        &self.reported_by
    }
    /// Returns the aggregate reported at.
    pub fn reported_at(&self) -> DateTime<Utc> {
        self.reported_at
    }
    /// Returns the aggregate summary.
    pub fn summary(&self) -> &incident::Summary {
        &self.summary
    }
    /// Returns the aggregate required review gates.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }
    /// Reports whether the incident is still active enough to require manager attention.
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self.status,
            incident::Status::NeedsManagerReview | incident::Status::LegalHold
        ) || matches!(
            self.severity,
            incident::Severity::High | incident::Severity::Critical
        ) || self
            .required_review_gates
            .contains(&policy::ReviewGate::ManagerApproval)
    }
}

#[derive(Debug, Clone, Default)]
/// Relationship-checked incident builder used at this boundary.
pub struct IncidentBuilder {
    id: Option<IncidentId>,
    location_id: Option<LocationId>,
    primary_subject: Option<IncidentSubject>,
    category: Option<incident::Category>,
    severity: Option<incident::Severity>,
    status: Option<incident::Status>,
    reported_by: Option<ActorRef>,
    reported_at: Option<DateTime<Utc>>,
    summary: Option<incident::Summary>,
    required_review_gates: Vec<policy::ReviewGate>,
    audit_refs: Vec<crate::audit::EventId>,
}
impl IncidentBuilder {
    /// Returns the aggregate id.
    pub fn id(mut self, value: IncidentId) -> Self {
        self.id = Some(value);
        self
    }
    /// Returns the aggregate location id.
    pub fn location_id(mut self, value: LocationId) -> Self {
        self.location_id = Some(value);
        self
    }
    /// Returns the aggregate primary subject.
    pub fn primary_subject(mut self, value: IncidentSubject) -> Self {
        self.primary_subject = Some(value);
        self
    }
    /// Returns the aggregate category.
    pub fn category(mut self, value: incident::Category) -> Self {
        self.category = Some(value);
        self
    }
    /// Returns the aggregate severity.
    pub fn severity(mut self, value: incident::Severity) -> Self {
        self.severity = Some(value);
        self
    }
    /// Returns the aggregate status.
    pub fn status(mut self, value: incident::Status) -> Self {
        self.status = Some(value);
        self
    }
    /// Returns the aggregate reported by.
    pub fn reported_by(mut self, value: ActorRef) -> Self {
        self.reported_by = Some(value);
        self
    }
    /// Returns the aggregate reported at.
    pub fn reported_at(mut self, value: DateTime<Utc>) -> Self {
        self.reported_at = Some(value);
        self
    }
    /// Returns the aggregate summary.
    pub fn summary(mut self, value: incident::Summary) -> Self {
        self.summary = Some(value);
        self
    }
    /// Returns the aggregate required review gates.
    pub fn required_review_gates(mut self, value: Vec<policy::ReviewGate>) -> Self {
        self.required_review_gates = value;
        self
    }
    /// Validates the accumulated fields and builds the aggregate.
    pub fn build(self) -> std::result::Result<Incident, IncidentError> {
        RawIncident {
            id: self.id.ok_or(IncidentError::IdRequired)?,
            location_id: self.location_id.ok_or(IncidentError::LocationIdRequired)?,
            primary_subject: self
                .primary_subject
                .ok_or(IncidentError::PrimarySubjectRequired)?,
            category: self.category.ok_or(IncidentError::CategoryRequired)?,
            severity: self.severity.ok_or(IncidentError::SeverityRequired)?,
            status: self.status.ok_or(IncidentError::StatusRequired)?,
            reported_by: self.reported_by.ok_or(IncidentError::ReportedByRequired)?,
            reported_at: self.reported_at.ok_or(IncidentError::ReportedAtRequired)?,
            summary: self.summary.ok_or(IncidentError::SummaryRequired)?,
            required_review_gates: self.required_review_gates,
            closure_evidence: None,
            audit_refs: self.audit_refs,
        }
        .try_into_incident()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject affected by an incident.
pub enum IncidentSubject {
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Resort location record participating in the workflow.
    Location(LocationId),
}
