//! Explicit codecs and projections for review-queue storage rows.
//!
//! Storage-to-app promotion is fallible where persisted facts require semantic
//! validation. App-to-storage projection is infallible because accepted app
//! objects already carry domain validation.

use app::data_quality_hygiene as hygiene;
use domain::{data_quality, entities, policy, source};

use crate::{
    read_model::{
        BlockedActionNoticeRow, HygieneOutcomeCardRow, ManagerQueueItemRow, StaffQueueItemRow,
    },
    storage::review_queue::{
        BlockedActionAttemptRow, BlockedActionReasonColumn, HygieneOutcomeRow,
        ResolutionStatusColumn, ReviewQueueItemRow, ReviewQueueStatusColumn,
        status_column::FeedbackOutcomeColumn,
    },
};

/// Current schema version for review-queue storage rows created by this adapter.
pub const REVIEW_QUEUE_SCHEMA_VERSION: u32 = 1;

/// Promotes a private storage row into the app review queue item.
pub fn review_queue_item(row: &ReviewQueueItemRow) -> Option<hygiene::ReviewQueueItem> {
    let action_id = hygiene::ActionId::try_new(row.action_id.clone()).ok()?;
    let location_id = crate::authz::parse_location_id(&row.location_id)?;
    let required_review_gates = if row.requires_manager_approval {
        vec![policy::ReviewGate::ManagerApproval]
    } else {
        Vec::new()
    };
    Some(hygiene::ReviewQueueItem::new(
        action_id,
        location_id,
        required_review_gates,
    ))
}

/// Projects a private queue row into the staff subscription read model.
pub fn staff_queue_item(row: &ReviewQueueItemRow) -> StaffQueueItemRow {
    StaffQueueItemRow {
        action_id: row.action_id.clone(),
        location_id: row.location_id.clone(),
        actor_id: row.actor_id.clone(),
        claimed_by_actor_id: row.claimed_by_actor_id.clone(),
        status_label: status_label(row.status).to_owned(),
        source_ref_id: row.source_ref_id.clone(),
        issue_ref: row.issue_ref.clone(),
        recommendation: row.recommendation.clone(),
        created_at: row.created_at,
        updated_at: row.updated_at,
        schema_version: row.schema_version,
    }
}

/// Projects manager-gated queue rows into the manager subscription read model.
pub fn manager_queue_item(row: &ReviewQueueItemRow) -> Option<ManagerQueueItemRow> {
    row.requires_manager_approval.then(|| ManagerQueueItemRow {
        action_id: row.action_id.clone(),
        location_id: row.location_id.clone(),
        actor_id: row.actor_id.clone(),
        claimed_by_actor_id: row.claimed_by_actor_id.clone(),
        requires_manager_approval: row.requires_manager_approval,
        status_label: status_label(row.status).to_owned(),
        source_ref_id: row.source_ref_id.clone(),
        issue_ref: row.issue_ref.clone(),
        recommendation: row.recommendation.clone(),
        staff_disposition: row.staff_disposition.clone(),
        manager_outcome: row.manager_outcome.clone(),
        created_at: row.created_at,
        updated_at: row.updated_at,
        schema_version: row.schema_version,
    })
}

/// Projects a blocked side-effect attempt into the public notice read model.
pub fn blocked_action_notice(row: &BlockedActionAttemptRow) -> BlockedActionNoticeRow {
    BlockedActionNoticeRow {
        id: row.id,
        action_id: row.action_id.clone(),
        actor_id: row.actor_id.clone(),
        location_id: row.location_id.clone(),
        attempted_side_effect: row.attempted_side_effect.clone(),
        reason_label: blocked_reason_label(row.reason).to_owned(),
        created_at: row.created_at,
        schema_version: row.schema_version,
    }
}

/// Stable status label used by subscription read models.
pub const fn status_label(status: ReviewQueueStatusColumn) -> &'static str {
    match status {
        ReviewQueueStatusColumn::PendingStaffReview => "pending_staff_review",
        ReviewQueueStatusColumn::ClaimedByStaff => "claimed_by_staff",
        ReviewQueueStatusColumn::PendingManagerApproval => "pending_manager_approval",
        ReviewQueueStatusColumn::ManagerApproved => "manager_approved",
        ReviewQueueStatusColumn::OutcomeRecorded => "outcome_recorded",
        ReviewQueueStatusColumn::Blocked => "blocked",
    }
}

/// Stable blocked-reason label used by subscription read models.
pub const fn blocked_reason_label(reason: BlockedActionReasonColumn) -> &'static str {
    match reason {
        BlockedActionReasonColumn::ActorNotFound => "actor_not_found",
        BlockedActionReasonColumn::ReviewQueueItemNotFound => "review_queue_item_not_found",
        BlockedActionReasonColumn::ActorLacksReviewGate => "actor_lacks_review_gate",
    }
}

/// Projects an accepted app outcome into a private persisted outcome row.
pub fn hygiene_outcome_row(outcome: &hygiene::OutcomeRecord, now: u64) -> HygieneOutcomeRow {
    HygieneOutcomeRow {
        action_id: outcome.action_id().as_ref().to_owned(),
        recorded_by: actor_ref_label(outcome.recorded_by()),
        outcome: feedback_outcome_column(outcome.outcome()),
        before_minutes: outcome.before_minutes().get().into(),
        actual_minutes: outcome.actual_minutes().get().into(),
        source_record_refs: encode_source_refs(outcome.source_record_refs()),
        issue_refs: encode_issue_refs(outcome.issue_refs()),
        reviewed_resolution_status: outcome
            .reviewed_resolution_status()
            .map(resolution_status_column),
        created_at: now,
        updated_at: now,
        schema_version: REVIEW_QUEUE_SCHEMA_VERSION,
    }
}

/// Projects an outcome storage row into the staff dashboard read model.
pub fn staff_outcome_card(row: HygieneOutcomeRow) -> HygieneOutcomeCardRow {
    HygieneOutcomeCardRow::new(
        row.action_id,
        row.recorded_by,
        format!("{:?}", row.outcome),
        row.before_minutes,
        row.actual_minutes,
        row.source_record_refs,
        row.issue_refs,
    )
}

/// Converts a reducer input column into the app feedback outcome.
pub const fn feedback_outcome(outcome: FeedbackOutcomeColumn) -> hygiene::FeedbackOutcome {
    match outcome {
        FeedbackOutcomeColumn::Completed => hygiene::FeedbackOutcome::Completed,
        FeedbackOutcomeColumn::Deferred => hygiene::FeedbackOutcome::Deferred,
        FeedbackOutcomeColumn::SuppressedByManager => hygiene::FeedbackOutcome::SuppressedByManager,
        FeedbackOutcomeColumn::SourceFactWasWrong => hygiene::FeedbackOutcome::SourceFactWasWrong,
        FeedbackOutcomeColumn::NotActionable => hygiene::FeedbackOutcome::NotActionable,
    }
}

/// Converts a reducer input column into the domain resolution status.
pub const fn resolution_status(status: ResolutionStatusColumn) -> data_quality::ResolutionStatus {
    match status {
        ResolutionStatusColumn::Open => data_quality::ResolutionStatus::Open,
        ResolutionStatusColumn::Acknowledged => data_quality::ResolutionStatus::Acknowledged,
        ResolutionStatusColumn::Ignored => data_quality::ResolutionStatus::Ignored,
        ResolutionStatusColumn::Repaired => data_quality::ResolutionStatus::Repaired,
        ResolutionStatusColumn::Superseded => data_quality::ResolutionStatus::Superseded,
    }
}

/// Converts app feedback outcome into its storage column representation.
pub fn feedback_outcome_column(outcome: hygiene::FeedbackOutcome) -> FeedbackOutcomeColumn {
    match outcome {
        hygiene::FeedbackOutcome::Completed => FeedbackOutcomeColumn::Completed,
        hygiene::FeedbackOutcome::Deferred => FeedbackOutcomeColumn::Deferred,
        hygiene::FeedbackOutcome::SuppressedByManager => FeedbackOutcomeColumn::SuppressedByManager,
        hygiene::FeedbackOutcome::SourceFactWasWrong => FeedbackOutcomeColumn::SourceFactWasWrong,
        hygiene::FeedbackOutcome::NotActionable => FeedbackOutcomeColumn::NotActionable,
    }
}

/// Converts domain resolution status into its storage column representation.
pub fn resolution_status_column(status: data_quality::ResolutionStatus) -> ResolutionStatusColumn {
    match status {
        data_quality::ResolutionStatus::Open => ResolutionStatusColumn::Open,
        data_quality::ResolutionStatus::Acknowledged => ResolutionStatusColumn::Acknowledged,
        data_quality::ResolutionStatus::Ignored => ResolutionStatusColumn::Ignored,
        data_quality::ResolutionStatus::Repaired => ResolutionStatusColumn::Repaired,
        data_quality::ResolutionStatus::Superseded => ResolutionStatusColumn::Superseded,
    }
}

/// Converts app blocked-action reason into its storage column representation.
pub fn blocked_reason_column(reason: hygiene::BlockedActionReason) -> BlockedActionReasonColumn {
    match reason {
        hygiene::BlockedActionReason::ActorNotFound => BlockedActionReasonColumn::ActorNotFound,
        hygiene::BlockedActionReason::ReviewQueueItemNotFound => {
            BlockedActionReasonColumn::ReviewQueueItemNotFound
        }
        hygiene::BlockedActionReason::ActorLacksReviewGate => {
            BlockedActionReasonColumn::ActorLacksReviewGate
        }
    }
}

/// Promotes primitive reducer minutes into app labor minutes.
pub fn labor_minutes(value: u32) -> Result<hygiene::LaborMinutes, String> {
    let value = u16::try_from(value).map_err(|_| "labor minutes exceed u16 range".to_owned())?;
    hygiene::LaborMinutes::try_new(value).map_err(|err| err.to_string())
}

/// Adapter label for actor refs used in storage/read-model rows.
pub fn actor_ref_label(actor: &entities::ActorRef) -> String {
    match actor {
        entities::ActorRef::Customer(id) => format!("customer:{}", id.0),
        entities::ActorRef::Staff { staff_id } => format!("staff:{staff_id:?}"),
        entities::ActorRef::Manager { manager_id } => format!("manager:{manager_id:?}"),
        entities::ActorRef::System => "system".to_owned(),
        entities::ActorRef::Agent { .. } => "agent".to_owned(),
    }
}

/// Encodes source refs for compact adapter read-model projection.
pub fn encode_source_refs(refs: Vec<&source::RecordRef>) -> String {
    refs.into_iter()
        .map(|record_ref| {
            format!(
                "{:?}:{}",
                record_ref.system(),
                record_ref.record_id().as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Encodes issue refs for compact adapter read-model projection.
pub fn encode_issue_refs(refs: Vec<&hygiene::IssueRef>) -> String {
    refs.into_iter()
        .map(|issue_ref| issue_ref.as_str().to_owned())
        .collect::<Vec<_>>()
        .join(",")
}

/// Initial review-queue status for a row inserted by upstream workflow adapters.
pub const fn initial_status(requires_manager_approval: bool) -> ReviewQueueStatusColumn {
    if requires_manager_approval {
        ReviewQueueStatusColumn::PendingManagerApproval
    } else {
        ReviewQueueStatusColumn::PendingStaffReview
    }
}
