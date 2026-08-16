//! Private SpacetimeDB rows for the Data-Quality Hygiene review queue.
//!
//! These rows intentionally flatten indexed/queryable facts such as action id,
//! actor id, location id, status-like columns, source refs, and schema version.
//! They are storage facts for reducers and projections, not embedded app/domain
//! objects.

#![allow(
    missing_docs,
    reason = "SpacetimeDB table macros generate public accessors/traits without rustdoc hooks; row structs and fields remain documented."
)]

use super::status_column::{
    ActorRefColumn, BlockedActionColumn, BlockedActionReasonColumn, FeedbackOutcomeColumn,
    IssueRefColumn, ManagerOutcomeColumn, RecommendationColumn, ResolutionStatusColumn,
    ReviewGateColumn, SourceRecordRefColumn, StaffDispositionColumn,
};

/// Review queue metadata needed before an outcome may be captured.
#[spacetimedb::table(accessor = review_queue_item)]
#[derive(Clone)]
pub struct ReviewQueueItemRow {
    /// Action id from the app workflow packet.
    #[primary_key]
    pub action_id: String,
    /// Location id that scopes the review gate. Demo fixtures may use compact ids like `101`.
    #[index(btree)]
    pub location_id: String,
    /// Actor expected to review this item when pre-assigned by upstream workflow.
    #[index(btree)]
    pub actor_id: Option<String>,
    /// Actor that claimed the item through the realtime queue.
    #[index(btree)]
    pub claimed_by_actor_id: Option<String>,
    /// Current queue status as an adapter column kept flat for subscriptions.
    pub status: ReviewQueueStatusColumn,
    /// Optional source record ref for subscription filtering/display.
    #[index(btree)]
    pub source_ref: Option<SourceRecordRefColumn>,
    /// Data-quality issue ref for traceable outcome capture.
    #[index(btree)]
    pub issue_ref: String,
    /// Staff/agent recommendation text; internal review draft only.
    pub recommendation: Option<RecommendationColumn>,
    /// Staff disposition text after triage.
    pub staff_disposition: Option<StaffDispositionColumn>,
    /// Manager disposition text when approval closes the gate.
    pub manager_outcome: Option<ManagerOutcomeColumn>,
    /// Unix timestamp when the item entered the queue.
    #[index(btree)]
    pub created_at: u64,
    /// Unix timestamp when reducer/projection state last changed.
    pub updated_at: u64,
    /// Exact review gates required before capture.
    pub required_review_gates: Vec<ReviewGateColumn>,
    /// Schema version for additive row evolution.
    pub schema_version: u32,
}

/// Workflow status encoded in review-queue storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum ReviewQueueStatusColumn {
    /// Waiting for staff review.
    PendingStaffReview,
    /// Claimed by a scoped staff actor.
    ClaimedByStaff,
    /// Staff triage is complete and manager approval is pending.
    PendingManagerApproval,
    /// Staff or manager disposition completed and an outcome may be captured.
    ReadyForOutcome,
    /// Accepted outcome has been recorded.
    OutcomeRecorded,
    /// Capture attempt failed closed.
    Blocked,
}

impl ReviewQueueStatusColumn {
    pub const ALL: [Self; 6] = [
        Self::PendingStaffReview,
        Self::ClaimedByStaff,
        Self::PendingManagerApproval,
        Self::ReadyForOutcome,
        Self::OutcomeRecorded,
        Self::Blocked,
    ];
}

/// Source-quality issue storage fact that feeds the workflow queue.
#[spacetimedb::table(accessor = data_quality_issue)]
#[derive(Clone)]
pub struct DataQualityIssueRow {
    /// Stable source-quality issue reference.
    #[primary_key]
    pub issue_ref: String,
    /// Location id affected by the issue.
    #[index(btree)]
    pub location_id: String,
    /// Source record ref that produced the issue.
    #[index(btree)]
    pub source_ref: SourceRecordRefColumn,
    /// Human-readable issue summary for queue projection.
    pub summary: String,
    /// Unix timestamp when the issue was stored.
    pub created_at: u64,
    /// Schema version for additive row evolution.
    pub schema_version: u32,
}

/// Append-only workflow transition event.
#[spacetimedb::table(accessor = workflow_event)]
#[derive(Clone)]
pub struct WorkflowEventRow {
    /// Synthetic append-only event id.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Action id whose workflow state changed.
    #[index(btree)]
    pub action_id: String,
    /// Actor id responsible for the transition.
    #[index(btree)]
    pub actor_id: String,
    /// Compact transition label.
    pub event_label: String,
    /// Human-readable transition detail.
    pub detail: String,
    /// Unix timestamp when the event was recorded.
    pub created_at: u64,
    /// Schema version for additive event evolution.
    pub schema_version: u32,
}

/// Workflow outcome summary row for manager/staff disposition history.
#[spacetimedb::table(accessor = workflow_outcome)]
#[derive(Clone)]
pub struct WorkflowOutcomeRow {
    /// Action id whose workflow outcome was recorded.
    #[primary_key]
    pub action_id: String,
    /// Actor id that recorded the outcome.
    #[index(btree)]
    pub actor_id: String,
    /// Display-safe workflow outcome label.
    pub outcome: ManagerOutcomeColumn,
    /// Unix timestamp when the outcome was recorded.
    pub created_at: u64,
    /// Schema version for additive outcome evolution.
    pub schema_version: u32,
}

/// Caller-reported outcome evidence after app-owned authorization accepts capture.
#[spacetimedb::table(accessor = hygiene_outcome)]
#[derive(Clone)]
pub struct HygieneOutcomeRow {
    /// Action id accepted through the app service.
    #[primary_key]
    pub action_id: String,
    /// Domain actor display payload preserved from the accepted outcome.
    #[index(btree)]
    pub recorded_by: ActorRefColumn,
    /// Caller-reported outcome label; this does not prove review or completion.
    pub outcome: FeedbackOutcomeColumn,
    /// Estimated pre-cleanup minutes.
    pub before_minutes: u32,
    /// Caller-reported minute evidence; this does not prove measured labor.
    pub actual_minutes: u32,
    /// Source record refs encoded for read-model projection.
    pub source_record_refs: Vec<SourceRecordRefColumn>,
    /// Data-quality issue refs encoded for read-model projection.
    pub issue_refs: Vec<IssueRefColumn>,
    /// Optional caller-reported resolution label; this does not prove review or resolution.
    pub reviewed_resolution_status: Option<ResolutionStatusColumn>,
    /// Unix timestamp when the outcome was persisted by the adapter.
    pub created_at: u64,
    /// Unix timestamp when this row shape was last updated.
    pub updated_at: u64,
    /// Schema version for additive row evolution.
    pub schema_version: u32,
}

/// Append-only audit event for accepted capture.
#[spacetimedb::table(accessor = hygiene_audit_event)]
#[derive(Clone)]
pub struct HygieneAuditEventRow {
    /// Synthetic audit id.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Accepted action id.
    #[index(btree)]
    pub action_id: String,
    /// Accountable actor payload.
    #[index(btree)]
    pub actor_id: String,
    /// Accountable actor label for audit display.
    pub actor: ActorRefColumn,
    /// Protected actions still blocked by the runtime.
    pub blocked_actions: Vec<BlockedActionColumn>,
    /// Unix timestamp when the audit event was projected.
    pub created_at: u64,
    /// Schema version for additive row evolution.
    pub schema_version: u32,
}

/// Fail-closed audit row for rejected capture attempts or forbidden side effects.
#[spacetimedb::table(accessor = blocked_action_attempt)]
#[derive(Clone)]
pub struct BlockedActionAttemptRow {
    /// Synthetic blocked-attempt id.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Action id submitted by the caller.
    #[index(btree)]
    pub action_id: String,
    /// App actor id submitted by the caller.
    #[index(btree)]
    pub actor_id: String,
    /// Location id associated with the attempted workflow action.
    #[index(btree)]
    pub location_id: String,
    /// Forbidden side effect the caller attempted.
    pub attempted_side_effect: BlockedActionColumn,
    /// Fail-closed reason returned by app authorization.
    pub reason: BlockedActionReasonColumn,
    /// Unix timestamp when the blocked event was recorded.
    pub created_at: u64,
    /// Schema version for additive row evolution.
    pub schema_version: u32,
}

macro_rules! redacted_debug {
    ($($row:ty => $name:literal),+ $(,)?) => {
        $(
            impl std::fmt::Debug for $row {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(concat!($name, "([REDACTED])"))
                }
            }
        )+
    };
}

redacted_debug!(
    ReviewQueueItemRow => "ReviewQueueItemRow",
    DataQualityIssueRow => "DataQualityIssueRow",
    WorkflowEventRow => "WorkflowEventRow",
    WorkflowOutcomeRow => "WorkflowOutcomeRow",
    HygieneOutcomeRow => "HygieneOutcomeRow",
    HygieneAuditEventRow => "HygieneAuditEventRow",
    BlockedActionAttemptRow => "BlockedActionAttemptRow",
);
