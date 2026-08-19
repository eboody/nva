use super::*;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Caller-supplied correlation identifiers used to project one reported Data-Quality Hygiene outcome into durable local-demo rows.
///
/// These serializable labels do not authenticate review, approval, identity, or execution.
pub struct DataQualityHygieneLineageIds {
    /// Durable workflow event id supplied by the repository adapter before insert.
    pub workflow_event_id: String,
    /// Durable review packet id supplied by the repository adapter before insert.
    pub review_packet_id: String,
    /// Durable approval record id supplied by the repository adapter before insert.
    pub approval_record_id: String,
    /// Durable outbox record id supplied by the repository adapter before insert.
    pub outbox_record_id: String,
    /// Location or other subject id used by the local-demo workflow event row.
    pub subject_id: String,
    /// Idempotency key used by the workflow event row and derived outbox candidate.
    pub idempotency_key: String,
    /// Stable timestamp copied into row projections for deterministic tests and replay.
    pub recorded_at: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Caller-supplied correlation identifiers used to project reported workflow output into approval/outbox rows.
///
/// These serializable labels do not themselves prove review or authorize an outbox operation.
pub struct ApprovalOutboxLineageIds {
    /// Durable workflow event id supplied by the repository adapter before insert.
    pub workflow_event_id: String,
    /// Durable review packet id supplied by the repository adapter before insert.
    pub review_packet_id: String,
    /// Durable approval record id supplied by the repository adapter before insert.
    pub approval_record_id: String,
    /// Durable outbox record id supplied by the repository adapter before insert.
    pub outbox_record_id: String,
    /// Subject family persisted for workflow/review/approval rows.
    pub subject_kind: String,
    /// Subject id used by workflow/review/approval/outbox rows.
    pub subject_id: String,
    /// Idempotency key used by the workflow event row and derived outbox candidate.
    pub idempotency_key: String,
    /// Stable timestamp copied into row projections for deterministic tests and replay.
    pub recorded_at: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Workflow-specific values used by the shared approval/outbox projection.
///
/// The input deliberately keeps workflow names, topics, payloads, and actor ids explicit
/// so the shared infrastructure does not erase service or workstream semantics.
pub struct ApprovalOutboxProjectionInput {
    /// Semantic workflow name persisted in `workflow_events.workflow_name`.
    pub workflow_name: String,
    /// Event kind persisted in `workflow_events.event_kind`.
    pub event_kind: String,
    /// Review gate used for packet, approval, and outbox rows.
    pub gate: ReviewGateCode,
    /// Target aggregate kind matching the approved outbox candidate.
    pub target_kind: String,
    /// Agent actor id that prepared the packet and requested approval.
    pub agent_actor_id: String,
    /// Workflow payload for source refs, correlation evidence, and safety posture.
    pub workflow_payload: serde_json::Value,
    /// Reviewable result payload; never execution proof for live side effects.
    pub result_payload: serde_json::Value,
    /// Audit-action label correlated to the reported-outcome row.
    pub audit_action: String,
    /// Caller-supplied audit metadata retained for inspection; it does not prove review, identity, source validity, or side-effect posture.
    pub audit_metadata: serde_json::Value,
    /// Closed topic and exact payload reviewed for a possible internal handoff.
    pub internal_handoff: InternalHandoff,
}

impl fmt::Debug for ApprovalOutboxProjectionInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApprovalOutboxProjectionInput([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Closed set of internal-only handoff topics accepted by storage and SQL.
pub enum InternalHandoffTopic {
    /// Reviewed data-quality work handed to an internal queue.
    DataQualityHygieneReviewedHandoff,
    /// Reviewed site-finance recommendation handed to an internal queue.
    SiteFinanceReviewedHandoff,
}

impl InternalHandoffTopic {
    /// Returns the exact stable SQL topic code.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DataQualityHygieneReviewedHandoff => {
                "internal.data_quality_hygiene.reviewed_handoff"
            }
            Self::SiteFinanceReviewedHandoff => "internal.site_finance.reviewed_handoff",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Exact closed topic and JSON payload presented to approval review.
pub struct InternalHandoff {
    topic: InternalHandoffTopic,
    payload: serde_json::Value,
}

impl fmt::Debug for InternalHandoff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("InternalHandoff([REDACTED])")
    }
}

impl InternalHandoff {
    /// Binds a reviewable payload to one closed internal topic.
    pub const fn new(topic: InternalHandoffTopic, payload: serde_json::Value) -> Self {
        Self { topic, payload }
    }

    /// Returns the closed internal handoff topic.
    pub const fn topic(&self) -> InternalHandoffTopic {
        self.topic
    }

    /// Returns the exact review-bound payload.
    pub const fn payload(&self) -> &serde_json::Value {
        &self.payload
    }
}

#[derive(PartialEq, Eq)]
/// Current authenticated reviewer capability projected into the storage boundary.
///
/// No production constructor is exposed. Live admission therefore remains
/// unrepresentable until a trusted authentication adapter owns capability issuance;
/// persisted approval evidence is deliberately not convertible into this capability.
pub struct CurrentApprovalReviewerCapability {
    actor_kind: ActorKindCode,
    actor_id: String,
}

impl CurrentApprovalReviewerCapability {
    /// Promotes a current staff/manager actor context into review capability.
    #[cfg(test)]
    pub(super) fn try_new(actor_kind: ActorKindCode, actor_id: String) -> Result<Self> {
        if !matches!(actor_kind, ActorKindCode::Staff | ActorKindCode::Manager) {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::CurrentRole,
            });
        }
        if actor_id.trim().is_empty() {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::CurrentActor,
            });
        }
        Ok(Self {
            actor_kind,
            actor_id,
        })
    }
}

#[derive(PartialEq, Eq)]
/// Pending storage row admitted by consuming one opaque approved internal-handoff authority.
pub struct PendingOutboxRecord {
    /// Outbox primary key.
    pub(super) id: String,
    /// Idempotency key for the candidate.
    pub(super) idempotency_key: String,
    /// Matching approved approval record id.
    pub(super) approval_record_id: String,
    /// Closed internal topic; customer/provider/payment/schedule topics are unrepresentable.
    pub(super) topic: InternalHandoffTopic,
    /// Review gate matching the approval row.
    pub(super) review_gate: ReviewGateCode,
    /// Aggregate kind matching the approval row.
    pub(super) aggregate_kind: String,
    /// Aggregate id matching the approval row.
    pub(super) aggregate_id: String,
    /// Candidate payload for local/internal handoff.
    pub(super) payload: serde_json::Value,
    /// Candidate status, fixed privately at admission.
    status: OutboxStatusCode,
    /// Availability timestamp.
    pub(super) available_at: String,
}

impl PendingOutboxRecord {
    /// Consumes one opaque authority to create exactly one pending persistence row.
    pub fn admit(authority: ApprovedInternalHandoffAuthority) -> Self {
        authority.pending
    }

    /// Durable identifier for the persisted internal handoff candidate.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Approval record whose decision admitted this candidate to the outbox.
    pub fn approval_record_id(&self) -> &str {
        &self.approval_record_id
    }

    /// Closed internal-only topic retained by the persistence record.
    pub const fn topic(&self) -> InternalHandoffTopic {
        self.topic
    }

    /// Review gate retained by the persistence record.
    pub const fn review_gate(&self) -> ReviewGateCode {
        self.review_gate
    }

    /// Read-only payload retained as historical handoff evidence, not execution authority.
    pub const fn payload(&self) -> &serde_json::Value {
        &self.payload
    }

    /// Persisted candidate status.
    pub const fn status(&self) -> OutboxStatusCode {
        self.status
    }

    fn has_expected_identity(&self, expected: &PendingOutboxIdentity) -> bool {
        self.id == expected.id
            && self.idempotency_key == expected.idempotency_key
            && self.available_at == expected.available_at
    }

    fn belongs_to_approval(&self, binding: &ApprovalOutboxBindingRecord) -> bool {
        self.approval_record_id == binding.approval_record_id
            && self.review_gate == binding.review_gate
            && self.aggregate_kind == binding.aggregate_kind
            && self.aggregate_id == binding.aggregate_id
    }

    fn matches_reviewed_handoff(&self, handoff: &InternalHandoff) -> bool {
        self.topic == handoff.topic && self.payload == handoff.payload
    }
}

/// Opaque, non-clone, non-serializable, one-shot authority for pending outbox admission.
pub struct ApprovedInternalHandoffAuthority {
    pending: PendingOutboxRecord,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Persisted relationship between one approval row and the exact reviewed handoff.
pub struct ApprovalOutboxBindingRecord {
    approval_record_id: String,
    review_gate: ReviewGateCode,
    aggregate_kind: String,
    aggregate_id: String,
    handoff: InternalHandoff,
}

impl ApprovalOutboxBindingRecord {
    /// Approval row whose decision controls this binding.
    pub fn approval_record_id(&self) -> &str {
        &self.approval_record_id
    }

    /// Exact closed topic and payload presented during review.
    pub const fn handoff(&self) -> &InternalHandoff {
        &self.handoff
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PendingOutboxIdentity {
    id: String,
    idempotency_key: String,
    available_at: String,
}

#[derive(PartialEq, Eq)]
/// Complete storage projection for one caller-reported local Data-Quality Hygiene outcome; it proves no review, repair, or completion.
pub struct DataQualityHygieneLocalPersistenceRecords {
    /// Workflow event row.
    pub workflow_event: WorkflowEventRecord,
    /// Workflow result row.
    pub workflow_result: WorkflowResultRecord,
    /// Review packet row.
    pub review_packet: ReviewPacketRecord,
    /// Approval record row.
    pub approval_record: ApprovalRecordRow,
    /// Outcome row linked to workflow and approval rows.
    pub outcome: DataQualityHygieneOutcomeRow,
    /// Append-only audit rows for context creation and reported-outcome admission.
    pub audit_events: Vec<AuditEventRecord>,
    /// Optional approved internal handoff candidate.
    pub outbox_candidate: Option<PendingOutboxRecord>,
}

#[derive(PartialEq, Eq)]
/// Complete storage projection for one reviewed local site-finance workflow outcome.
pub struct SiteFinanceLocalPersistenceRecords {
    /// Workflow event row.
    pub workflow_event: WorkflowEventRecord,
    /// Workflow result row.
    pub workflow_result: WorkflowResultRecord,
    /// Review packet row.
    pub review_packet: ReviewPacketRecord,
    /// Approval record row.
    pub approval_record: ApprovalRecordRow,
    /// Outcome row linked to workflow and approval rows.
    pub outcome: SiteFinanceOutcomeRow,
    /// Append-only audit rows for context creation and reported-outcome admission.
    pub audit_events: Vec<AuditEventRecord>,
    /// Optional approved internal handoff candidate.
    pub outbox_candidate: Option<PendingOutboxRecord>,
}

#[derive(PartialEq, Eq)]
/// Complete shared storage projection for a reviewed local workflow handoff.
///
/// This is the common approval/outbox spine proven by multiple vertical slices. Outcome
/// payload rows remain workflow-owned so this infrastructure does not become a generic
/// platform type that hides service or workstream semantics.
pub struct ApprovalOutboxProjection {
    /// Workflow event row.
    pub workflow_event: WorkflowEventRecord,
    /// Workflow result row.
    pub workflow_result: WorkflowResultRecord,
    /// Review packet row.
    pub review_packet: ReviewPacketRecord,
    /// Approval record row.
    pub approval_record: ApprovalRecordRow,
    /// Append-only audit rows for context creation and reported-outcome admission.
    pub audit_events: Vec<AuditEventRecord>,
    /// Exact topic/payload relation persisted before any admission is attempted.
    pub outbox_binding: ApprovalOutboxBindingRecord,
    /// Optional admitted pending handoff. The field remains private to prevent replacement.
    outbox_candidate: Option<PendingOutboxRecord>,
    expected_pending_outbox_identity: PendingOutboxIdentity,
    pub(super) admission_authority_available: bool,
}

impl ApprovalOutboxProjection {
    /// Projects caller-reported handoff evidence into pending review rows.
    ///
    /// Serializable disposition, actor, target, gate, and timestamp labels are normalized to
    /// pending. They cannot mint completion, approval, reviewer attribution, or outbox authority.
    pub fn from_reviewed_internal_handoff(
        ids: ApprovalOutboxLineageIds,
        input: ApprovalOutboxProjectionInput,
    ) -> Self {
        let internal_handoff = input.internal_handoff.clone();
        let workflow_event = WorkflowEventRecord {
            id: ids.workflow_event_id.clone(),
            workflow_name: input.workflow_name.clone(),
            event_kind: input.event_kind,
            subject_kind: ids.subject_kind.clone(),
            subject_id: ids.subject_id.clone(),
            idempotency_key: ids.idempotency_key.clone(),
            payload: input.workflow_payload.clone(),
            occurred_at: ids.recorded_at.clone(),
            recorded_at: ids.recorded_at.clone(),
        };

        let workflow_result = WorkflowResultRecord {
            id: format!("{}:result", ids.workflow_event_id),
            workflow_event_id: ids.workflow_event_id.clone(),
            status: WorkflowResultStatusCode::NeedsReview,
            result: input.result_payload,
            error_code: None,
            created_at: ids.recorded_at.clone(),
        };

        let review_packet = ReviewPacketRecord {
            id: ids.review_packet_id.clone(),
            subject_kind: ids.subject_kind.clone(),
            subject_id: ids.subject_id.clone(),
            gate: input.gate,
            status: ReviewPacketStatusCode::ReadyForReview,
            workflow_event_id: ids.workflow_event_id.clone(),
            created_by_actor_kind: ActorKindCode::Agent,
            created_by_actor_id: input.agent_actor_id.clone(),
            created_at: ids.recorded_at.clone(),
            updated_at: ids.recorded_at.clone(),
        };

        let approval_record = ApprovalRecordRow {
            id: ids.approval_record_id.clone(),
            target_kind: input.target_kind.clone(),
            target_id: ids.subject_id.clone(),
            gate: input.gate,
            status: "approval_requested".to_owned(),
            requested_by_actor_kind: ActorKindCode::Agent,
            requested_by_actor_id: input.agent_actor_id.clone(),
            requested_at: ids.recorded_at.clone(),
            decided_by_actor_kind: None,
            decided_by_actor_id: None,
            decided_at: None,
            review_packet_id: ids.review_packet_id.clone(),
        };

        let audit_events = vec![
            AuditEventRecord {
                actor_kind: ActorKindCode::Agent,
                actor_id: input.agent_actor_id.clone(),
                subject_kind: "workflow_event".to_owned(),
                subject_id: ids.workflow_event_id.clone(),
                action: format!("{}.context_recorded", input.workflow_name.replace('-', "_")),
                workflow_event_id: ids.workflow_event_id.clone(),
                metadata: input.workflow_payload,
                occurred_at: ids.recorded_at.clone(),
                recorded_at: ids.recorded_at.clone(),
            },
            AuditEventRecord {
                actor_kind: ActorKindCode::Agent,
                actor_id: input.agent_actor_id.clone(),
                subject_kind: "approval".to_owned(),
                subject_id: ids.approval_record_id.clone(),
                action: input.audit_action,
                workflow_event_id: ids.workflow_event_id.clone(),
                metadata: input.audit_metadata,
                occurred_at: ids.recorded_at.clone(),
                recorded_at: ids.recorded_at.clone(),
            },
        ];

        let outbox_binding = ApprovalOutboxBindingRecord {
            approval_record_id: ids.approval_record_id.clone(),
            review_gate: input.gate,
            aggregate_kind: input.target_kind,
            aggregate_id: ids.subject_id,
            handoff: internal_handoff,
        };
        let expected_pending_outbox_identity = PendingOutboxIdentity {
            id: ids.outbox_record_id,
            idempotency_key: format!("{}:internal-reviewed-handoff", ids.idempotency_key),
            available_at: ids.recorded_at,
        };

        Self {
            workflow_event,
            workflow_result,
            review_packet,
            approval_record,
            audit_events,
            outbox_binding,
            outbox_candidate: None,
            expected_pending_outbox_identity,
            admission_authority_available: false,
        }
    }

    /// Issues one opaque admission authority only when every current and persisted fact matches.
    pub fn authorize_internal_handoff(
        &mut self,
        current_reviewer: &CurrentApprovalReviewerCapability,
        requested_handoff: InternalHandoff,
    ) -> Result<ApprovedInternalHandoffAuthority> {
        if self.approval_record.status != "approved" {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::ApprovalStatus,
            });
        }
        if !reviewer_role_authorizes_gate(current_reviewer.actor_kind, self.approval_record.gate) {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::CurrentRole,
            });
        }
        if self.approval_record.decided_by_actor_kind != Some(current_reviewer.actor_kind)
            || self.approval_record.decided_by_actor_id.as_deref()
                != Some(current_reviewer.actor_id.as_str())
        {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::CurrentActor,
            });
        }
        if self.outbox_binding.approval_record_id != self.approval_record.id
            || self.outbox_binding.review_gate != self.approval_record.gate
            || self.outbox_binding.aggregate_kind != self.approval_record.target_kind
            || self.outbox_binding.aggregate_id != self.approval_record.target_id
        {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::ApprovalRelation,
            });
        }
        if requested_handoff != self.outbox_binding.handoff {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::InternalHandoff,
            });
        }
        if !self.admission_authority_available {
            return Err(Error::ApprovalOutboxAlreadyAdmitted);
        }
        self.admission_authority_available = false;
        let identity = &self.expected_pending_outbox_identity;
        Ok(ApprovedInternalHandoffAuthority {
            pending: PendingOutboxRecord {
                id: identity.id.clone(),
                idempotency_key: identity.idempotency_key.clone(),
                approval_record_id: self.outbox_binding.approval_record_id.clone(),
                topic: self.outbox_binding.handoff.topic,
                review_gate: self.outbox_binding.review_gate,
                aggregate_kind: self.outbox_binding.aggregate_kind.clone(),
                aggregate_id: self.outbox_binding.aggregate_id.clone(),
                payload: self.outbox_binding.handoff.payload.clone(),
                status: OutboxStatusCode::Pending,
                available_at: identity.available_at.clone(),
            },
        })
    }

    /// Returns an admitted pending row without exposing replacement or publishing operations.
    pub const fn outbox_candidate(&self) -> Option<&PendingOutboxRecord> {
        self.outbox_candidate.as_ref()
    }

    /// Records the pending row produced by consuming this projection's one-shot authority.
    pub fn record_pending_outbox(&mut self, pending: PendingOutboxRecord) -> Result<()> {
        if self.outbox_candidate.is_some() {
            return Err(Error::ApprovalOutboxAlreadyAdmitted);
        }
        if !pending.has_expected_identity(&self.expected_pending_outbox_identity)
            || !pending.belongs_to_approval(&self.outbox_binding)
        {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::ApprovalRelation,
            });
        }
        if !pending.matches_reviewed_handoff(&self.outbox_binding.handoff) {
            return Err(Error::ApprovalOutboxAuthority {
                reason: ApprovalOutboxAuthorityMismatch::InternalHandoff,
            });
        }
        self.outbox_candidate = Some(pending);
        Ok(())
    }
}

const fn reviewer_role_authorizes_gate(actor_kind: ActorKindCode, gate: ReviewGateCode) -> bool {
    match gate {
        ReviewGateCode::ManagerApproval
        | ReviewGateCode::CustomerMessageApproval
        | ReviewGateCode::RefundOrDepositException => matches!(actor_kind, ActorKindCode::Manager),
        ReviewGateCode::MedicalDocumentReview | ReviewGateCode::BehaviorReview => {
            matches!(actor_kind, ActorKindCode::Staff | ActorKindCode::Manager)
        }
    }
}
impl SiteFinanceLocalPersistenceRecords {
    /// Projects caller-reported site-finance evidence into pending storage-shaped MVP rows.
    ///
    /// Projection proves no review, authorization, completion, action, measurement, or value.
    pub fn from_reported_outcome(
        ids: ApprovalOutboxLineageIds,
        outcome: SiteFinanceOutcomeRecord,
    ) -> Self {
        let workflow_completion = outcome.workflow_completion;
        let value_attribution = outcome.value_attribution;
        let projection = ApprovalOutboxProjection::from_reviewed_internal_handoff(
            ids.clone(),
            ApprovalOutboxProjectionInput::builder()
                .workflow_name("site-finance".to_owned())
                .event_kind("reported_recommendation_recorded".to_owned())
                .gate(ReviewGateCode::ManagerApproval)
                .target_kind("message".to_owned())
                .agent_actor_id("site-finance-agent".to_owned())
                .workflow_payload(json!({
                    "correlation_id": outcome.correlation_id,
                    "location_id": outcome.location_id,
                    "recommendation_id": outcome.recommendation_id,
                    "review_packet_id": outcome.review_packet_id,
                    "audit_event_id": outcome.audit_event_id,
                    "source_refs": outcome.source_refs,
                    "payment_actions_allowed": false,
                    "accounting_mutations_allowed": false,
                }))
                .result_payload(json!({
                    "record_only_finance_action": true,
                    "legal_action": outcome.legal_action,
                    "value_attribution": value_attribution.to_string(),
                    "workflow_completion": workflow_completion.to_string(),
                    "can_support_value_claim": value_attribution.can_support_value_claim(),
                    "live_side_effects_allowed": false,
                }))
                .audit_action("site_finance.reported_recommendation_recorded".to_owned())
                .audit_metadata(json!({
                    "recommendation_id": outcome.recommendation_id,
                    "legal_action": outcome.legal_action,
                    "value_attribution": value_attribution.to_string(),
                    "workflow_completion": workflow_completion.to_string(),
                    "can_support_value_claim": value_attribution.can_support_value_claim(),
                    "payment_actions_allowed": false,
                    "accounting_mutations_allowed": false,
                }))
                .internal_handoff(InternalHandoff::new(
                    InternalHandoffTopic::SiteFinanceReviewedHandoff,
                    json!({
                        "recommendation_id": outcome.recommendation_id,
                        "correlation_id": outcome.correlation_id,
                        "source_refs": outcome.source_refs,
                        "internal_handoff_only": true,
                        "live_delivery_allowed": false,
                    }),
                ))
                .build(),
        );
        let mut workflow_result = projection.workflow_result;
        workflow_result.status = workflow_completion.workflow_result_status();

        Self {
            workflow_event: projection.workflow_event,
            workflow_result,
            review_packet: projection.review_packet,
            approval_record: projection.approval_record,
            outcome: SiteFinanceOutcomeRow {
                workflow_event_id: ids.workflow_event_id,
                approval_record_id: ids.approval_record_id,
                record: outcome,
            },
            audit_events: projection.audit_events,
            outbox_candidate: None,
        }
    }
}


impl DataQualityHygieneLocalPersistenceRecords {
    /// Projects caller-reported Data Quality Hygiene evidence into pending storage rows without proving review, repair, completion, labor, or value.
    pub fn from_reported_outcome(
        ids: DataQualityHygieneLineageIds,
        outcome: DataQualityHygieneOutcomeRecord,
    ) -> Self {
        let workflow_payload = json!({
            "correlation_id": outcome.correlation_id,
            "location_id": outcome.location_id,
            "operating_day": outcome.operating_day,
            "action_id": outcome.action_id,
            "source_refs": outcome.source_refs,
            "issue_refs": outcome.issue_refs,
            "live_side_effects_allowed": false,
            "provider_writes_allowed": false,
            "customer_messages_allowed": false,
        });
        let projection = ApprovalOutboxProjection::from_reviewed_internal_handoff(
            ApprovalOutboxLineageIds::builder()
                .workflow_event_id(ids.workflow_event_id.clone())
                .review_packet_id(ids.review_packet_id.clone())
                .approval_record_id(ids.approval_record_id.clone())
                .outbox_record_id(ids.outbox_record_id)
                .subject_kind("location".to_owned())
                .subject_id(ids.subject_id)
                .idempotency_key(ids.idempotency_key)
                .recorded_at(ids.recorded_at)
                .build(),
            ApprovalOutboxProjectionInput::builder()
                .workflow_name("data-quality-hygiene".to_owned())
                .event_kind("context_created".to_owned())
                .gate(ReviewGateCode::ManagerApproval)
                .target_kind("message".to_owned())
                .agent_actor_id("data-quality-hygiene-agent".to_owned())
                .workflow_payload(workflow_payload)
                .result_payload(json!({
                    "mode": "fake_deterministic_or_disabled",
                    "reviewable_output_only": true,
                    "action_id": outcome.action_id,
                    "outcome": outcome.outcome,
                    "live_side_effects_allowed": false,
                }))
                .audit_action("data_quality_hygiene.reported_outcome_recorded".to_owned())
                .audit_metadata(json!({
                    "action_id": outcome.action_id,
                    "outcome": outcome.outcome,
                    "reported_resolution_status": outcome.reported_resolution_status,
                    "reported_estimated_minutes_difference": outcome.reported_estimated_minutes_difference,
                    "live_side_effects_allowed": false,
                }))
                .internal_handoff(InternalHandoff::new(
                    InternalHandoffTopic::DataQualityHygieneReviewedHandoff,
                    json!({
                        "action_id": outcome.action_id,
                        "correlation_id": outcome.correlation_id,
                        "issue_refs": outcome.issue_refs,
                        "source_refs": outcome.source_refs,
                        "internal_handoff_only": true,
                        "live_delivery_allowed": false,
                    }),
                ))
                .build(),
        );
        // Every caller-serializable outcome remains review evidence. In particular, the
        // historical `Completed` label cannot manufacture an executable success result.
        let workflow_result = projection.workflow_result;

        Self {
            workflow_event: projection.workflow_event,
            workflow_result,
            review_packet: projection.review_packet,
            approval_record: projection.approval_record,
            outcome: DataQualityHygieneOutcomeRow {
                workflow_event_id: ids.workflow_event_id,
                approval_record_id: ids.approval_record_id,
                record: outcome,
            },
            audit_events: projection.audit_events,
            outbox_candidate: None,
        }
    }
}
