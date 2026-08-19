//! Disposable loopback-only real-host coverage module.
//!
//! This crate is intentionally outside the production workspace. It links the exact production
//! SpacetimeDB reducer crate, drives those reducer functions through a real host during module init,
//! and exposes raw LLVM profile bytes only from this synthetic module's schema.

use app::data_quality_hygiene as hygiene;
use nva_spacetimedb::{
    reducers,
    runtime::HygieneCaptureRuntime,
    storage::review_queue::{
        BlockedActionColumn, FeedbackOutcomeColumn, ManagerOutcomeColumn, ResolutionStatusColumn,
        ReviewGateColumn, StaffDispositionColumn, codec,
    },
    tables::{
        ActorKindColumn, LocationScopeRow, ReviewerRoleColumn, StaffActorRow,
        blocked_action_attempt, hygiene_outcome, location_scope, review_queue_item, staff_actor,
    },
};
use spacetimedb::{ReducerContext, Table};

#[spacetimedb::table(accessor = wasm_coverage_snapshot, public)]
pub struct WasmCoverageSnapshotRow {
    #[primary_key]
    pub id: u8,
    pub profile: Vec<u8>,
}

#[spacetimedb::reducer(init)]
pub fn initialize_wasm_coverage(ctx: &ReducerContext) -> Result<(), String> {
    if !nva_spacetimedb::__link_wasm_coverage_runtime() {
        return Err("production reducer crate was not instrumented for wasm coverage".to_owned());
    }
    ctx.db.staff_actor().insert(StaffActorRow {
        actor_id: "coverage-manager".to_owned(),
        identity: ctx.sender().to_string(),
        actor_kind: ActorKindColumn::Manager,
        actor_ref: "stale-coverage-manager-ref".to_owned(),
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    reducers::seed_demo_actor(
        ctx,
        "coverage-manager".to_owned(),
        ctx.sender().to_string(),
        ActorKindColumn::Manager,
        "coverage-manager-ref".to_owned(),
        ReviewerRoleColumn::GeneralManager,
        "101".to_owned(),
    )?;
    reducers::seed_demo_actor(
        ctx,
        "coverage-inserted-manager".to_owned(),
        "coverage-inserted-identity".to_owned(),
        ActorKindColumn::Manager,
        "coverage-inserted-manager-ref".to_owned(),
        ReviewerRoleColumn::GeneralManager,
        "101".to_owned(),
    )?;

    seed_issue(ctx, "success", vec![ReviewGateColumn::ManagerApproval])?;
    complete_manager_reviewed_outcome(ctx, "success")?;
    seed_issue(ctx, "success-two", vec![ReviewGateColumn::ManagerApproval])?;
    complete_manager_reviewed_outcome(ctx, "success-two")?;

    seed_issue(
        ctx,
        "invalid-transition",
        vec![ReviewGateColumn::ManagerApproval],
    )?;
    if reducers::attach_recommendation(
        ctx,
        action_id("invalid-transition"),
        "cannot attach before claim".to_owned(),
    )
    .is_ok()
    {
        return Err("invalid transition unexpectedly succeeded".to_owned());
    }

    seed_issue_at(
        ctx,
        "scope-upgrade",
        "202",
        vec![ReviewGateColumn::ManagerApproval],
    )?;
    reducers::attempt_blocked_side_effect(
        ctx,
        action_id("scope-upgrade"),
        BlockedActionColumn::SendCustomerMessage,
    )?;
    ctx.db.location_scope().insert(LocationScopeRow {
        id: 0,
        actor_id: "coverage-manager".to_owned(),
        location_id: "202".to_owned(),
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    reducers::claim_review_item(ctx, action_id("scope-upgrade"))?;

    seed_issue_at(
        ctx,
        "unauthorized-work",
        "303",
        vec![ReviewGateColumn::ManagerApproval],
    )?;
    let unauthorized_work_action = action_id("unauthorized-work");
    let unauthorized_work_before = ctx
        .db
        .review_queue_item()
        .action_id()
        .find(unauthorized_work_action.clone())
        .ok_or_else(|| "unauthorized work fixture row was not persisted".to_owned())?;
    reducers::claim_review_item(ctx, unauthorized_work_action.clone())?;
    reducers::attach_recommendation(
        ctx,
        unauthorized_work_action.clone(),
        "must not attach without scope".to_owned(),
    )?;
    reducers::record_staff_disposition(
        ctx,
        unauthorized_work_action.clone(),
        StaffDispositionColumn::RecommendForManagerApproval,
    )?;
    reducers::record_manager_outcome(
        ctx,
        unauthorized_work_action.clone(),
        ManagerOutcomeColumn::Approved,
    )?;
    let unauthorized_work_after = ctx
        .db
        .review_queue_item()
        .action_id()
        .find(unauthorized_work_action)
        .ok_or_else(|| "unauthorized work fixture row disappeared".to_owned())?;
    if unauthorized_work_after.status != unauthorized_work_before.status
        || unauthorized_work_after.actor_id != unauthorized_work_before.actor_id
        || unauthorized_work_after.claimed_by_actor_id
            != unauthorized_work_before.claimed_by_actor_id
        || unauthorized_work_after.recommendation != unauthorized_work_before.recommendation
        || unauthorized_work_after.staff_disposition != unauthorized_work_before.staff_disposition
        || unauthorized_work_after.manager_outcome != unauthorized_work_before.manager_outcome
        || unauthorized_work_after.updated_at != unauthorized_work_before.updated_at
    {
        return Err("unauthorized queue work mutated the protected review row".to_owned());
    }

    seed_issue(ctx, "blocked", vec![ReviewGateColumn::ManagerApproval])?;
    reducers::claim_review_item(ctx, action_id("blocked"))?;
    reducers::attempt_blocked_side_effect(
        ctx,
        action_id("blocked"),
        BlockedActionColumn::SendCustomerMessage,
    )?;

    seed_issue(
        ctx,
        "unauthorized-outcome",
        vec![ReviewGateColumn::MedicalDocumentReview],
    )?;
    let unauthorized_action = action_id("unauthorized-outcome");
    let outcome_count_before = ctx.db.hygiene_outcome().count();
    let blocked_count_before = ctx.db.blocked_action_attempt().count();
    let row_before = ctx
        .db
        .review_queue_item()
        .action_id()
        .find(unauthorized_action.clone())
        .ok_or_else(|| "unauthorized fixture row was not persisted".to_owned())?;
    reducers::record_reported_hygiene_outcome(
        ctx,
        unauthorized_action.clone(),
        FeedbackOutcomeColumn::Deferred,
        20,
        20,
        None,
    )?;
    let row_after = ctx
        .db
        .review_queue_item()
        .action_id()
        .find(unauthorized_action.clone())
        .ok_or_else(|| "unauthorized fixture row disappeared".to_owned())?;
    if ctx.db.hygiene_outcome().count() != outcome_count_before
        || row_after.status != row_before.status
        || row_after.updated_at != row_before.updated_at
        || ctx.db.blocked_action_attempt().count() != blocked_count_before + 1
    {
        return Err("unauthorized scoped actor mutated protected outcome state".to_owned());
    }
    drive_runtime_blocked_persistence(ctx, unauthorized_action)?;

    Ok(())
}

fn complete_manager_reviewed_outcome(ctx: &ReducerContext, suffix: &str) -> Result<(), String> {
    reducers::claim_review_item(ctx, action_id(suffix))?;
    reducers::attach_recommendation(ctx, action_id(suffix), "review source record".to_owned())?;
    reducers::record_staff_disposition(
        ctx,
        action_id(suffix),
        StaffDispositionColumn::RecommendForManagerApproval,
    )?;
    reducers::record_manager_outcome(ctx, action_id(suffix), ManagerOutcomeColumn::Approved)?;
    reducers::record_reported_hygiene_outcome(
        ctx,
        action_id(suffix),
        FeedbackOutcomeColumn::Completed,
        30,
        10,
        Some(ResolutionStatusColumn::Repaired),
    )
}

fn drive_runtime_blocked_persistence(
    ctx: &ReducerContext,
    action_id: String,
) -> Result<(), String> {
    let outcome = hygiene::OutcomeRecord::builder()
        .action_id(hygiene::ActionId::try_new(action_id).map_err(|error| error.to_string())?)
        .recorded_by(domain::entities::ActorRef::Manager {
            manager_id: domain::entities::ManagerId::try_new("coverage-manager-ref")
                .map_err(|error| error.to_string())?,
        })
        .outcome(hygiene::FeedbackOutcome::Deferred)
        .before_minutes(hygiene::LaborMinutes::try_new(20).map_err(|error| error.to_string())?)
        .actual_minutes(hygiene::LaborMinutes::try_new(20).map_err(|error| error.to_string())?)
        .source_record_refs(vec![domain::source::RecordRef::new(
            domain::source::System::ManualImport,
            domain::source::record::Id::try_new("coverage-source-unauthorized-outcome")
                .map_err(|error| error.to_string())?,
        )])
        .issue_refs(vec![
            hygiene::IssueRef::try_new("coverage-issue-unauthorized-outcome")
                .map_err(|error| error.to_string())?,
        ])
        .build()
        .map_err(|error| error.to_string())?;
    let request = hygiene::OutcomeCaptureRequest::new(
        hygiene::ActorId::try_new("coverage-manager").map_err(|error| error.to_string())?,
        outcome,
    );
    if HygieneCaptureRuntime::load(ctx)
        .record_reported_outcome(ctx, request)
        .is_ok()
    {
        return Err("runtime accepted an outcome without the row-required review gate".to_owned());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn capture_wasm_coverage(ctx: &ReducerContext) -> Result<(), String> {
    if reducers::seed_demo_actor(
        ctx,
        "coverage-external-actor".to_owned(),
        ctx.sender().to_string(),
        ActorKindColumn::Manager,
        "coverage-external-manager-ref".to_owned(),
        ReviewerRoleColumn::GeneralManager,
        "101".to_owned(),
    )
    .is_ok()
    {
        return Err("external actor fixture seed unexpectedly succeeded".to_owned());
    }
    if reducers::seed_demo_issue(
        ctx,
        "coverage-external-seed".to_owned(),
        "coverage-external-action".to_owned(),
        "101".to_owned(),
        "coverage-external-source".to_owned(),
        "external seed must be rejected".to_owned(),
        vec![ReviewGateColumn::ManagerApproval],
    )
    .is_ok()
    {
        return Err("external fixture seed unexpectedly succeeded".to_owned());
    }

    let mut profile = Vec::new();
    // SAFETY: the disposable loopback harness invokes this reducer after init has completed, with
    // no concurrent reducer calls mutating minicov's process-global counters.
    unsafe { minicov::capture_coverage(&mut profile) }.map_err(|error| error.to_string())?;
    ctx.db
        .wasm_coverage_snapshot()
        .insert(WasmCoverageSnapshotRow { id: 0, profile });
    Ok(())
}

fn seed_issue(
    ctx: &ReducerContext,
    suffix: &str,
    required_review_gates: Vec<ReviewGateColumn>,
) -> Result<(), String> {
    seed_issue_at(ctx, suffix, "101", required_review_gates)
}

fn seed_issue_at(
    ctx: &ReducerContext,
    suffix: &str,
    location_id: &str,
    required_review_gates: Vec<ReviewGateColumn>,
) -> Result<(), String> {
    reducers::seed_demo_issue(
        ctx,
        format!("coverage-issue-{suffix}"),
        action_id(suffix),
        location_id.to_owned(),
        format!("coverage-source-{suffix}"),
        format!("coverage summary {suffix}"),
        required_review_gates,
    )
}

fn action_id(suffix: &str) -> String {
    format!("coverage-action-{suffix}")
}
