use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Actor kinds accepted by MVP workflow, approval, audit, and outbox tables.
pub enum ActorKindCode {
    /// Customer actor persisted at the storage boundary.
    Customer,
    /// Staff actor persisted at the storage boundary.
    Staff,
    /// Manager actor persisted at the storage boundary.
    Manager,
    /// System actor persisted at the storage boundary.
    System,
    /// Agent actor persisted at the storage boundary.
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Review gates accepted by the MVP review/approval/outbox migration.
pub enum ReviewGateCode {
    /// Manager approval gate for internal handoff and data-quality review.
    ManagerApproval,
    /// Medical document review gate.
    MedicalDocumentReview,
    /// Behavior review gate.
    BehaviorReview,
    /// Customer message approval gate.
    CustomerMessageApproval,
    /// Refund or deposit exception gate.
    RefundOrDepositException,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Review packet statuses accepted by the MVP review packet table.
pub enum ReviewPacketStatusCode {
    /// Draft packet not yet ready for review.
    Draft,
    /// Packet prepared for review.
    ReadyForReview,
    /// Packet under review.
    InReview,
    /// Packet approved by the appropriate review gate.
    Approved,
    /// Packet rejected by review.
    Rejected,
    /// Packet cancelled before completion.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Workflow result statuses accepted by the MVP workflow results table.
pub enum WorkflowResultStatusCode {
    /// Workflow completed locally without enabling live side effects.
    Succeeded,
    /// Workflow failed before reviewable output.
    Failed,
    /// Workflow produced reviewable output that still needs review.
    NeedsReview,
    /// Workflow was deferred.
    Deferred,
    /// Workflow was cancelled.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Outbox statuses accepted by the MVP outbox table.
pub enum OutboxStatusCode {
    /// Candidate is available for local/internal review processing only.
    Pending,
    /// Candidate was claimed by a worker.
    Claimed,
    /// Candidate was published by a future approved adapter.
    Published,
    /// Candidate failed but may retry.
    Failed,
    /// Candidate failed permanently.
    DeadLetter,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped workflow event row for the local Data-Quality Hygiene demo slice.
pub struct WorkflowEventRecord {
    /// Workflow event primary key.
    pub id: String,
    /// Semantic workflow name persisted in `workflow_events.workflow_name`.
    pub workflow_name: String,
    /// Event kind persisted in `workflow_events.event_kind`.
    pub event_kind: String,
    /// Subject family persisted in `workflow_events.subject_kind`.
    pub subject_kind: String,
    /// Subject id persisted in `workflow_events.subject_id`.
    pub subject_id: String,
    /// Idempotency key persisted in `workflow_events.idempotency_key`.
    pub idempotency_key: String,
    /// JSON payload for source refs, issue refs, correlation evidence, and safety posture.
    pub payload: serde_json::Value,
    /// Event occurrence timestamp.
    pub occurred_at: String,
    /// Storage record timestamp.
    pub recorded_at: String,
}

impl fmt::Debug for WorkflowEventRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WorkflowEventRecord([REDACTED])")
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped workflow result row for reviewable fake/deterministic output.
pub struct WorkflowResultRecord {
    /// Workflow result primary key or derived local identifier.
    pub id: String,
    /// Parent workflow event id.
    pub workflow_event_id: String,
    /// Result status accepted by `workflow_results.status`.
    pub status: WorkflowResultStatusCode,
    /// Reviewable result payload; never execution proof for live side effects.
    pub result: serde_json::Value,
    /// Optional local error code for failed results.
    pub error_code: Option<String>,
    /// Result creation timestamp.
    pub created_at: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped review packet row for the manager/front-desk review gate.
pub struct ReviewPacketRecord {
    /// Review packet primary key.
    pub id: String,
    /// Subject family persisted for review.
    pub subject_kind: String,
    /// Subject id persisted for review.
    pub subject_id: String,
    /// Review gate required before handoff.
    pub gate: ReviewGateCode,
    /// Review packet status.
    pub status: ReviewPacketStatusCode,
    /// Linked workflow event id.
    pub workflow_event_id: String,
    /// Actor kind that prepared the packet.
    pub created_by_actor_kind: ActorKindCode,
    /// Actor id that prepared the packet.
    pub created_by_actor_id: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Update timestamp.
    pub updated_at: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped approval row for a reviewed local handoff candidate.
pub struct ApprovalRecordRow {
    /// Approval primary key.
    pub id: String,
    /// Target aggregate kind matching the outbox candidate.
    pub target_kind: String,
    /// Target aggregate id matching the outbox candidate.
    pub target_id: String,
    /// Review gate used for the decision.
    pub gate: ReviewGateCode,
    /// Approval status string accepted by the migration.
    pub status: String,
    /// Actor kind that requested approval.
    pub requested_by_actor_kind: ActorKindCode,
    /// Actor id that requested approval.
    pub requested_by_actor_id: String,
    /// Request timestamp.
    pub requested_at: String,
    /// Actor kind that decided approval, present only for approved/rejected rows.
    pub decided_by_actor_kind: Option<ActorKindCode>,
    /// Actor id that decided approval, present only for approved/rejected rows.
    pub decided_by_actor_id: Option<String>,
    /// Decision timestamp, present only for approved/rejected rows.
    pub decided_at: Option<String>,
    /// Linked review packet id.
    pub review_packet_id: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped Data-Quality Hygiene outcome row with workflow and approval foreign keys.
pub struct DataQualityHygieneOutcomeRow {
    /// Parent workflow event id.
    pub workflow_event_id: String,
    /// Parent approval record id.
    pub approval_record_id: String,
    /// Typed storage outcome payload.
    pub record: DataQualityHygieneOutcomeRecord,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped site-finance outcome row with workflow and approval foreign keys.
pub struct SiteFinanceOutcomeRow {
    /// Parent workflow event id.
    pub workflow_event_id: String,
    /// Parent approval record id.
    pub approval_record_id: String,
    /// Typed storage outcome payload.
    pub record: SiteFinanceOutcomeRecord,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage-shaped audit event row for append-only local proof.
pub struct AuditEventRecord {
    /// Actor kind that produced the audit event.
    pub actor_kind: ActorKindCode,
    /// Actor id that produced the audit event.
    pub actor_id: String,
    /// Audited subject family.
    pub subject_kind: String,
    /// Audited subject id.
    pub subject_id: String,
    /// Audit action.
    pub action: String,
    /// Linked workflow event id.
    pub workflow_event_id: String,
    /// Metadata proving source refs, issue refs, and side-effect posture.
    pub metadata: serde_json::Value,
    /// Event occurrence timestamp.
    pub occurred_at: String,
    /// Storage record timestamp.
    pub recorded_at: String,
}
