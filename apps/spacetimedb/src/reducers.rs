//! Thin SpacetimeDB reducer entrypoints.
//!
//! Reducers translate untrusted client primitives into app/domain values, invoke
//! app services through runtime adapters, and persist table/read-model/audit rows.
//! They do not contain business policy and they do not perform live external side
//! effects. Denied commands that need audit evidence persist blocked rows and return
//! `Ok(())` because returning `Err` would roll the SpacetimeDB transaction back.

#![allow(
    missing_docs,
    reason = "SpacetimeDB reducer macros generate public ABI structs without rustdoc hooks; reducer functions remain documented."
)]

use app::data_quality_hygiene as hygiene;
use domain::source;
use spacetimedb::{ReducerContext, Table};

use crate::{
    read_model::{
        manager_queue_item::manager_queue_item,
        staff_queue_item::{blocked_action_notice, hygiene_outcome_card, staff_queue_item},
    },
    runtime::HygieneCaptureRuntime,
    storage::review_queue::{
        BlockedActionAttemptRow, BlockedActionReasonColumn, DataQualityIssueRow,
        FeedbackOutcomeColumn, ResolutionStatusColumn, ReviewQueueItemRow, ReviewQueueStatusColumn,
        WorkflowEventRow, WorkflowOutcomeRow, codec,
    },
    tables::{
        ActorKindColumn, HygieneAuditEventRow, LocationScopeRow, ReviewerRoleColumn,
        RoleAssignmentRow, StaffActorRow, blocked_action_attempt, data_quality_issue,
        hygiene_audit_event, hygiene_outcome, location_scope, review_queue_item, role_assignment,
        staff_actor, workflow_event, workflow_outcome,
    },
};

/// Seeds a demo staff/manager actor plus role and location scope rows.
#[spacetimedb::reducer]
pub fn seed_demo_actor(
    ctx: &ReducerContext,
    actor_id: String,
    identity: String,
    actor_kind: ActorKindColumn,
    actor_ref: String,
    review_role: ReviewerRoleColumn,
    location_id: String,
) -> Result<(), String> {
    let actor = StaffActorRow {
        actor_id: actor_id.clone(),
        identity,
        actor_kind,
        actor_ref,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    };
    upsert_staff_actor(ctx, actor);
    ctx.db.role_assignment().insert(RoleAssignmentRow {
        id: 0,
        actor_id: actor_id.clone(),
        review_role,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    ctx.db.location_scope().insert(LocationScopeRow {
        id: 0,
        actor_id,
        location_id,
    });
    Ok(())
}

/// Seeds a source-quality issue and the corresponding location review queue item.
#[spacetimedb::reducer]
pub fn seed_demo_issue(
    ctx: &ReducerContext,
    issue_ref: String,
    action_id: String,
    location_id: String,
    source_ref_id: String,
    summary: String,
    requires_manager_approval: bool,
) -> Result<(), String> {
    ctx.db.data_quality_issue().insert(DataQualityIssueRow {
        issue_ref: issue_ref.clone(),
        location_id: location_id.clone(),
        source_ref_id: source_ref_id.clone(),
        summary: summary.clone(),
        created_at: 0,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    let row = ReviewQueueItemRow {
        action_id: action_id.clone(),
        location_id,
        actor_id: None,
        claimed_by_actor_id: None,
        status: codec::initial_status(requires_manager_approval),
        source_ref_id: Some(source_ref_id),
        issue_ref,
        recommendation: Some(summary),
        staff_disposition: None,
        manager_outcome: None,
        created_at: 0,
        updated_at: 0,
        requires_manager_approval,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    };
    upsert_review_queue_item(ctx, row.clone());
    append_workflow_event(ctx, &action_id, "system", "seed_demo_issue", "issue seeded");
    project_queue_item(ctx, &row);
    Ok(())
}

/// Claims a review queue item for the scoped staff actor represented by the caller identity.
#[spacetimedb::reducer]
pub fn claim_review_item(ctx: &ReducerContext, action_id: String) -> Result<(), String> {
    let mut row = review_queue_row(ctx, &action_id)?;
    let actor_id = actor_id_for_sender(ctx)?;
    if actor_authorized_for_queue_work(ctx, &actor_id, &row)?.is_none() {
        return Ok(());
    }
    row.claimed_by_actor_id = Some(actor_id.as_ref().to_owned());
    row.status = ReviewQueueStatusColumn::ClaimedByStaff;
    row.updated_at = row.updated_at.saturating_add(1);
    upsert_review_queue_item(ctx, row.clone());
    append_workflow_event(
        ctx,
        &action_id,
        actor_id.as_ref(),
        "claim_review_item",
        "claimed",
    );
    project_queue_item(ctx, &row);
    Ok(())
}

/// Attaches a staff/agent recommendation draft to the claimed queue item.
#[spacetimedb::reducer]
pub fn attach_recommendation(
    ctx: &ReducerContext,
    action_id: String,
    recommendation: String,
) -> Result<(), String> {
    let mut row = review_queue_row(ctx, &action_id)?;
    let actor_id = actor_id_for_sender(ctx)?;
    if actor_authorized_for_queue_work(ctx, &actor_id, &row)?.is_none() {
        return Ok(());
    }
    row.recommendation = Some(recommendation);
    row.updated_at = row.updated_at.saturating_add(1);
    upsert_review_queue_item(ctx, row.clone());
    append_workflow_event(
        ctx,
        &action_id,
        actor_id.as_ref(),
        "attach_recommendation",
        "recommendation attached",
    );
    project_queue_item(ctx, &row);
    Ok(())
}

/// Records staff triage disposition and routes manager-gated work to manager review.
#[spacetimedb::reducer]
pub fn record_staff_disposition(
    ctx: &ReducerContext,
    action_id: String,
    disposition: String,
) -> Result<(), String> {
    let mut row = review_queue_row(ctx, &action_id)?;
    let actor_id = actor_id_for_sender(ctx)?;
    if actor_authorized_for_queue_work(ctx, &actor_id, &row)?.is_none() {
        return Ok(());
    }
    row.staff_disposition = Some(disposition);
    row.status = if row.requires_manager_approval {
        ReviewQueueStatusColumn::PendingManagerApproval
    } else {
        ReviewQueueStatusColumn::OutcomeRecorded
    };
    row.updated_at = row.updated_at.saturating_add(1);
    upsert_review_queue_item(ctx, row.clone());
    append_workflow_event(
        ctx,
        &action_id,
        actor_id.as_ref(),
        "record_staff_disposition",
        "staff disposition recorded",
    );
    project_queue_item(ctx, &row);
    Ok(())
}

/// Records manager/regional disposition for manager-gated work.
#[spacetimedb::reducer]
pub fn record_manager_outcome(
    ctx: &ReducerContext,
    action_id: String,
    manager_outcome: String,
) -> Result<(), String> {
    let mut row = review_queue_row(ctx, &action_id)?;
    let actor_id = actor_id_for_sender(ctx)?;
    let Some(actor) = actor_authorized_for_row(ctx, &actor_id, &row)? else {
        return Ok(());
    };
    row.manager_outcome = Some(manager_outcome.clone());
    row.status = ReviewQueueStatusColumn::ManagerApproved;
    row.updated_at = row.updated_at.saturating_add(1);
    upsert_review_queue_item(ctx, row.clone());
    ctx.db.workflow_outcome().insert(WorkflowOutcomeRow {
        action_id: action_id.clone(),
        actor_id: actor_id.as_ref().to_owned(),
        outcome_label: manager_outcome,
        created_at: row.updated_at,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    ctx.db.hygiene_audit_event().insert(HygieneAuditEventRow {
        id: 0,
        action_id: action_id.clone(),
        actor_id: actor_id.as_ref().to_owned(),
        actor: codec::actor_ref_label(actor.actor()),
        blocked_actions: "live_customer_provider_side_effects_blocked".to_owned(),
        created_at: row.updated_at,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    append_workflow_event(
        ctx,
        &action_id,
        actor_id.as_ref(),
        "record_manager_outcome",
        "manager outcome recorded",
    );
    project_queue_item(ctx, &row);
    Ok(())
}

/// Records an attempted unsafe customer/provider side effect without performing it.
#[spacetimedb::reducer]
pub fn attempt_blocked_side_effect(
    ctx: &ReducerContext,
    action_id: String,
    attempted_side_effect: String,
) -> Result<(), String> {
    let mut row = review_queue_row(ctx, &action_id)?;
    let actor_id = actor_id_for_sender(ctx)?;
    let blocked = BlockedActionAttemptRow {
        id: 0,
        action_id: action_id.clone(),
        actor_id: actor_id.as_ref().to_owned(),
        location_id: row.location_id.clone(),
        attempted_side_effect,
        reason: BlockedActionReasonColumn::ActorLacksReviewGate,
        created_at: row.updated_at.saturating_add(1),
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    };
    ctx.db.blocked_action_attempt().insert(blocked.clone());
    ctx.db
        .blocked_action_notice()
        .try_insert(codec::blocked_action_notice(&blocked))
        .ok();
    row.status = ReviewQueueStatusColumn::Blocked;
    row.updated_at = blocked.created_at;
    upsert_review_queue_item(ctx, row.clone());
    append_workflow_event(
        ctx,
        &action_id,
        actor_id.as_ref(),
        "attempt_blocked_side_effect",
        "blocked unsafe side effect",
    );
    project_queue_item(ctx, &row);
    Ok(())
}

/// Captures a reviewed data-quality hygiene outcome through app-owned ports.
#[allow(
    clippy::too_many_arguments,
    reason = "SpacetimeDB reducer ABI exposes primitive client arguments; the body immediately converts them into app/domain request types."
)]
#[spacetimedb::reducer]
pub fn record_reviewed_hygiene_outcome(
    ctx: &ReducerContext,
    action_id: String,
    outcome: FeedbackOutcomeColumn,
    before_minutes: u32,
    actual_minutes: u32,
    source_record_id: String,
    issue_ref: String,
    reviewed_resolution_status: Option<ResolutionStatusColumn>,
) -> Result<(), String> {
    let actor_id = actor_id_for_sender(ctx)?;
    let Some(actor) =
        actor_authorized_for_row(ctx, &actor_id, &review_queue_row(ctx, &action_id)?)?
    else {
        return Ok(());
    };
    let mut outcome_builder = hygiene::OutcomeRecord::builder()
        .action_id(hygiene::ActionId::try_new(action_id.clone()).map_err(|err| err.to_string())?)
        .recorded_by(actor.actor().clone())
        .outcome(codec::feedback_outcome(outcome))
        .before_minutes(codec::labor_minutes(before_minutes)?)
        .actual_minutes(codec::labor_minutes(actual_minutes)?)
        .source_record_refs(vec![source::RecordRef::new(
            source::System::ManualImport,
            source::record::Id::try_new(source_record_id).map_err(|err| err.to_string())?,
        )])
        .issue_refs(vec![
            hygiene::IssueRef::try_new(issue_ref).map_err(|err| err.to_string())?,
        ]);
    if let Some(status) = reviewed_resolution_status {
        outcome_builder =
            outcome_builder.reviewed_resolution_status(codec::resolution_status(status));
    }
    let outcome_record = outcome_builder.build().map_err(|err| err.to_string())?;

    let request = hygiene::OutcomeCaptureRequest::new(actor_id, outcome_record);
    let runtime = HygieneCaptureRuntime::load(ctx);
    if runtime.record_reviewed_outcome(ctx, request).is_err() {
        return Ok(());
    }
    if let Ok(mut row) = review_queue_row(ctx, &action_id) {
        row.status = ReviewQueueStatusColumn::OutcomeRecorded;
        row.updated_at = row.updated_at.saturating_add(1);
        upsert_review_queue_item(ctx, row.clone());
        project_queue_item(ctx, &row);
    }
    project_latest_outcome_cards(ctx);
    Ok(())
}

fn actor_id_for_sender(ctx: &ReducerContext) -> Result<hygiene::ActorId, String> {
    HygieneCaptureRuntime::actor_id_for_sender(ctx)
        .ok_or_else(|| "sender identity did not resolve to an app actor".to_owned())
}

fn actor_authorized_for_queue_work(
    ctx: &ReducerContext,
    actor_id: &hygiene::ActorId,
    row: &ReviewQueueItemRow,
) -> Result<Option<hygiene::ActorAssignment>, String> {
    actor_authorized_for_row_action(ctx, actor_id, row, QueueAuthorizationAction::WorkItem)
}

fn actor_authorized_for_row(
    ctx: &ReducerContext,
    actor_id: &hygiene::ActorId,
    row: &ReviewQueueItemRow,
) -> Result<Option<hygiene::ActorAssignment>, String> {
    actor_authorized_for_row_action(ctx, actor_id, row, QueueAuthorizationAction::RecordOutcome)
}

enum QueueAuthorizationAction {
    WorkItem,
    RecordOutcome,
}

fn actor_authorized_for_row_action(
    ctx: &ReducerContext,
    actor_id: &hygiene::ActorId,
    row: &ReviewQueueItemRow,
    action: QueueAuthorizationAction,
) -> Result<Option<hygiene::ActorAssignment>, String> {
    use crate::adapter::ActorDirectoryAdapter;
    use app::data_quality_hygiene::{ActorDirectory, AuthorizationPolicy};

    let directory = ActorDirectoryAdapter::new(
        ctx.db.staff_actor().iter().collect::<Vec<StaffActorRow>>(),
        ctx.db
            .role_assignment()
            .iter()
            .collect::<Vec<RoleAssignmentRow>>(),
        ctx.db
            .location_scope()
            .iter()
            .collect::<Vec<LocationScopeRow>>(),
    );
    let actor = directory
        .resolve_actor(actor_id)
        .ok_or_else(|| "sender actor row could not be promoted into app actor".to_owned())?;
    let review_item = codec::review_queue_item(row)
        .ok_or_else(|| "review queue row could not be promoted into app review item".to_owned())?;
    let authorized = match action {
        QueueAuthorizationAction::WorkItem => {
            hygiene::RoleLocationAuthorization.can_work_queue_item(&actor, &review_item)
        }
        QueueAuthorizationAction::RecordOutcome => {
            hygiene::RoleLocationAuthorization.can_record_outcome(&actor, &review_item)
        }
    };
    if !authorized {
        record_blocked_attempt(
            ctx,
            row,
            actor_id.as_ref(),
            match action {
                QueueAuthorizationAction::WorkItem => "unauthorized_queue_work",
                QueueAuthorizationAction::RecordOutcome => "unauthorized_outcome_capture",
            },
            BlockedActionReasonColumn::ActorLacksReviewGate,
        );
        return Ok(None);
    }
    Ok(Some(actor))
}

fn review_queue_row(ctx: &ReducerContext, action_id: &str) -> Result<ReviewQueueItemRow, String> {
    ctx.db
        .review_queue_item()
        .action_id()
        .find(action_id.to_owned())
        .ok_or_else(|| "review queue item not found".to_owned())
}

fn upsert_staff_actor(ctx: &ReducerContext, row: StaffActorRow) {
    if ctx
        .db
        .staff_actor()
        .actor_id()
        .find(row.actor_id.clone())
        .is_some()
    {
        ctx.db.staff_actor().actor_id().update(row);
    } else {
        ctx.db.staff_actor().insert(row);
    }
}

fn upsert_review_queue_item(ctx: &ReducerContext, row: ReviewQueueItemRow) {
    if ctx
        .db
        .review_queue_item()
        .action_id()
        .find(row.action_id.clone())
        .is_some()
    {
        ctx.db.review_queue_item().action_id().update(row);
    } else {
        ctx.db.review_queue_item().insert(row);
    }
}

fn project_queue_item(ctx: &ReducerContext, row: &ReviewQueueItemRow) {
    let staff = codec::staff_queue_item(row);
    if ctx
        .db
        .staff_queue_item()
        .action_id()
        .find(staff.action_id.clone())
        .is_some()
    {
        ctx.db.staff_queue_item().action_id().update(staff);
    } else {
        ctx.db.staff_queue_item().insert(staff);
    }

    if let Some(manager) = codec::manager_queue_item(row) {
        if ctx
            .db
            .manager_queue_item()
            .action_id()
            .find(manager.action_id.clone())
            .is_some()
        {
            ctx.db.manager_queue_item().action_id().update(manager);
        } else {
            ctx.db.manager_queue_item().insert(manager);
        }
    }
}

fn record_blocked_attempt(
    ctx: &ReducerContext,
    row: &ReviewQueueItemRow,
    actor_id: &str,
    attempted_side_effect: &str,
    reason: BlockedActionReasonColumn,
) {
    let blocked = BlockedActionAttemptRow {
        id: 0,
        action_id: row.action_id.clone(),
        actor_id: actor_id.to_owned(),
        location_id: row.location_id.clone(),
        attempted_side_effect: attempted_side_effect.to_owned(),
        reason,
        created_at: row.updated_at.saturating_add(1),
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    };
    ctx.db.blocked_action_attempt().insert(blocked.clone());
    ctx.db
        .blocked_action_notice()
        .try_insert(codec::blocked_action_notice(&blocked))
        .ok();
}

fn append_workflow_event(
    ctx: &ReducerContext,
    action_id: &str,
    actor_id: &str,
    event_label: &str,
    detail: &str,
) {
    ctx.db.workflow_event().insert(WorkflowEventRow {
        id: 0,
        action_id: action_id.to_owned(),
        actor_id: actor_id.to_owned(),
        event_label: event_label.to_owned(),
        detail: detail.to_owned(),
        created_at: 0,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
}

fn project_latest_outcome_cards(ctx: &ReducerContext) {
    for row in ctx.db.hygiene_outcome().iter() {
        let card = codec::staff_outcome_card(row);
        if ctx
            .db
            .hygiene_outcome_card()
            .action_id()
            .find(card.action_id.clone())
            .is_some()
        {
            ctx.db.hygiene_outcome_card().action_id().update(card);
        } else {
            ctx.db.hygiene_outcome_card().insert(card);
        }
    }
}
