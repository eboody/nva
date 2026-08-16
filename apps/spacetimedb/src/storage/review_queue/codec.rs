//! Explicit codecs and projections for review-queue storage rows.
//!
//! Storage-to-app promotion is fallible where persisted facts require semantic
//! validation. App-to-storage projection is infallible because accepted app
//! objects already carry domain validation.

use app::data_quality_hygiene as hygiene;
use domain::{data_quality, entities, policy, source};

use crate::{
    read_model::{
        BlockedActionNoticeRow, HygieneOutcomeCardV1Row, ManagerQueueItemRow, StaffQueueItemRow,
    },
    storage::review_queue::{
        BlockedActionAttemptRow, BlockedActionReasonColumn, HygieneOutcomeRow,
        ResolutionStatusColumn, ReviewQueueItemRow, ReviewQueueStatusColumn,
        status_column::{
            ActorRefColumn, BlockedActionColumn, FeedbackOutcomeColumn, IssueRefColumn,
            ReviewGateColumn, SourceRecordRefColumn, SourceSystemColumn,
        },
    },
};

/// Current schema version for review-queue storage rows created by this adapter.
pub const REVIEW_QUEUE_SCHEMA_VERSION: u32 = 2;

/// Promotes a private storage row into the app review queue item.
pub fn review_queue_item(row: &ReviewQueueItemRow) -> Option<hygiene::ReviewQueueItem> {
    if row.schema_version != REVIEW_QUEUE_SCHEMA_VERSION {
        return None;
    }
    let action_id = hygiene::ActionId::try_new(row.action_id.clone()).ok()?;
    let location_id = crate::authz::parse_location_id(&row.location_id)?;
    let required_review_gates = row
        .required_review_gates
        .iter()
        .copied()
        .map(review_gate)
        .collect();
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
        source_ref: row.source_ref.clone(),
        issue_ref: row.issue_ref.clone(),
        recommendation: row.recommendation.clone(),
        created_at: row.created_at,
        updated_at: row.updated_at,
        schema_version: row.schema_version,
    }
}

/// Projects manager-gated queue rows into the manager subscription read model.
pub fn manager_queue_item(row: &ReviewQueueItemRow) -> Option<ManagerQueueItemRow> {
    row.required_review_gates
        .contains(&ReviewGateColumn::ManagerApproval)
        .then(|| ManagerQueueItemRow {
            action_id: row.action_id.clone(),
            location_id: row.location_id.clone(),
            actor_id: row.actor_id.clone(),
            claimed_by_actor_id: row.claimed_by_actor_id.clone(),
            required_review_gates: row.required_review_gates.clone(),
            status_label: status_label(row.status).to_owned(),
            source_ref: row.source_ref.clone(),
            issue_ref: row.issue_ref.clone(),
            recommendation: row.recommendation.clone(),
            staff_disposition: row.staff_disposition,
            manager_outcome: row.manager_outcome,
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
        attempted_side_effect: row.attempted_side_effect,
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
        ReviewQueueStatusColumn::ReadyForOutcome => "ready_for_outcome",
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
        recorded_by: actor_ref_column(outcome.recorded_by()),
        outcome: feedback_outcome_column(outcome.outcome()),
        before_minutes: outcome.before_minutes().get().into(),
        actual_minutes: outcome.actual_minutes().get().into(),
        source_record_refs: encode_source_refs(outcome.source_record_refs()),
        issue_refs: encode_issue_refs(outcome.issue_refs()),
        reviewed_resolution_status: outcome
            .reported_resolution_status()
            .map(resolution_status_column),
        created_at: now,
        updated_at: now,
        schema_version: REVIEW_QUEUE_SCHEMA_VERSION,
    }
}

/// Projects an outcome storage row into the staff dashboard read model.
pub fn staff_outcome_card(row: HygieneOutcomeRow) -> HygieneOutcomeCardV1Row {
    HygieneOutcomeCardV1Row::new(
        row.action_id,
        row.recorded_by,
        row.outcome,
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

/// Stable actor representation used in storage/read-model rows.
pub fn actor_ref_column(actor: &entities::ActorRef) -> ActorRefColumn {
    match actor {
        entities::ActorRef::Customer(id) => ActorRefColumn::Customer(id.get().to_string()),
        entities::ActorRef::Staff { staff_id } => {
            ActorRefColumn::Staff(staff_id.clone().into_inner())
        }
        entities::ActorRef::Manager { manager_id } => {
            ActorRefColumn::Manager(manager_id.clone().into_inner())
        }
        entities::ActorRef::System => ActorRefColumn::System,
        entities::ActorRef::Agent { workflow } => {
            ActorRefColumn::Agent(workflow.clone().into_inner())
        }
    }
}

/// Rehydrates a stable actor representation.
pub fn actor_ref(actor: &ActorRefColumn) -> Option<entities::ActorRef> {
    Some(match actor {
        ActorRefColumn::Customer(id) => {
            entities::ActorRef::Customer(entities::CustomerId::new(uuid::Uuid::parse_str(id).ok()?))
        }
        ActorRefColumn::Staff(id) => entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new(id.clone()).ok()?,
        },
        ActorRefColumn::Manager(id) => entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new(id.clone()).ok()?,
        },
        ActorRefColumn::System => entities::ActorRef::System,
        ActorRefColumn::Agent(workflow) => entities::ActorRef::Agent {
            workflow: domain::agent::Name::try_new(workflow.clone()).ok()?,
        },
    })
}

/// Encodes source refs for compact adapter read-model projection.
pub fn encode_source_refs(refs: Vec<&source::RecordRef>) -> Vec<SourceRecordRefColumn> {
    refs.into_iter()
        .map(|record_ref| SourceRecordRefColumn {
            system: source_system_column(record_ref.system()),
            record_id: record_ref.record_id().as_str().to_owned(),
        })
        .collect()
}

/// Encodes issue refs for compact adapter read-model projection.
pub fn encode_issue_refs(refs: Vec<&hygiene::IssueRef>) -> Vec<IssueRefColumn> {
    refs.into_iter()
        .map(|issue_ref| issue_ref.as_str().to_owned())
        .collect()
}

/// Initial review-queue status for a row inserted by upstream workflow adapters.
pub const fn initial_status() -> ReviewQueueStatusColumn {
    ReviewQueueStatusColumn::PendingStaffReview
}

/// Parses a stable review-queue status label.
pub fn parse_status(label: &str) -> Result<ReviewQueueStatusColumn, String> {
    ReviewQueueStatusColumn::ALL
        .into_iter()
        .find(|status| status_label(*status) == label)
        .ok_or_else(|| format!("unsupported review queue status: {label}"))
}

/// Promotes the stored review gate into the semantic application value.
pub const fn review_gate(gate: ReviewGateColumn) -> policy::ReviewGate {
    match gate {
        ReviewGateColumn::ManagerApproval => policy::ReviewGate::ManagerApproval,
        ReviewGateColumn::MedicalDocumentReview => policy::ReviewGate::MedicalDocumentReview,
        ReviewGateColumn::BehaviorReview => policy::ReviewGate::BehaviorReview,
        ReviewGateColumn::CustomerMessageApproval => policy::ReviewGate::CustomerMessageApproval,
        ReviewGateColumn::RefundOrDepositException => policy::ReviewGate::RefundOrDepositException,
    }
}

/// Projects review gate into its stable storage column.
pub const fn review_gate_column(gate: policy::ReviewGate) -> ReviewGateColumn {
    match gate {
        policy::ReviewGate::ManagerApproval => ReviewGateColumn::ManagerApproval,
        policy::ReviewGate::MedicalDocumentReview => ReviewGateColumn::MedicalDocumentReview,
        policy::ReviewGate::BehaviorReview => ReviewGateColumn::BehaviorReview,
        policy::ReviewGate::CustomerMessageApproval => ReviewGateColumn::CustomerMessageApproval,
        policy::ReviewGate::RefundOrDepositException => ReviewGateColumn::RefundOrDepositException,
    }
}

/// Promotes the stored source system into the semantic application value.
pub const fn source_system(system: SourceSystemColumn) -> source::System {
    match system {
        SourceSystemColumn::Gingr => source::System::Gingr,
        SourceSystemColumn::Telephony => source::System::Telephony,
        SourceSystemColumn::SmsProvider => source::System::SmsProvider,
        SourceSystemColumn::Email => source::System::Email,
        SourceSystemColumn::WebChat => source::System::WebChat,
        SourceSystemColumn::WebsiteForms => source::System::WebsiteForms,
        SourceSystemColumn::MarketingAutomation => source::System::MarketingAutomation,
        SourceSystemColumn::Crm => source::System::Crm,
        SourceSystemColumn::FinanceAccounting => source::System::FinanceAccounting,
        SourceSystemColumn::WorkforceManagement => source::System::WorkforceManagement,
        SourceSystemColumn::KnowledgeBase => source::System::KnowledgeBase,
        SourceSystemColumn::ProviderOrPms => source::System::ProviderOrPms,
        SourceSystemColumn::BusinessIntelligence => source::System::BusinessIntelligence,
        SourceSystemColumn::LaborScheduling => source::System::LaborScheduling,
        SourceSystemColumn::Timeclock => source::System::Timeclock,
        SourceSystemColumn::Payroll => source::System::Payroll,
        SourceSystemColumn::CapacityInventory => source::System::CapacityInventory,
        SourceSystemColumn::PointOfSale => source::System::PointOfSale,
        SourceSystemColumn::ManualImport => source::System::ManualImport,
    }
}

/// Projects source system into its stable storage column.
pub const fn source_system_column(system: source::System) -> SourceSystemColumn {
    match system {
        source::System::Gingr => SourceSystemColumn::Gingr,
        source::System::Telephony => SourceSystemColumn::Telephony,
        source::System::SmsProvider => SourceSystemColumn::SmsProvider,
        source::System::Email => SourceSystemColumn::Email,
        source::System::WebChat => SourceSystemColumn::WebChat,
        source::System::WebsiteForms => SourceSystemColumn::WebsiteForms,
        source::System::MarketingAutomation => SourceSystemColumn::MarketingAutomation,
        source::System::Crm => SourceSystemColumn::Crm,
        source::System::FinanceAccounting => SourceSystemColumn::FinanceAccounting,
        source::System::WorkforceManagement => SourceSystemColumn::WorkforceManagement,
        source::System::KnowledgeBase => SourceSystemColumn::KnowledgeBase,
        source::System::ProviderOrPms => SourceSystemColumn::ProviderOrPms,
        source::System::BusinessIntelligence => SourceSystemColumn::BusinessIntelligence,
        source::System::LaborScheduling => SourceSystemColumn::LaborScheduling,
        source::System::Timeclock => SourceSystemColumn::Timeclock,
        source::System::Payroll => SourceSystemColumn::Payroll,
        source::System::CapacityInventory => SourceSystemColumn::CapacityInventory,
        source::System::PointOfSale => SourceSystemColumn::PointOfSale,
        source::System::ManualImport => SourceSystemColumn::ManualImport,
    }
}

/// Promotes the stored blocked action into the semantic application value.
pub const fn blocked_action(action: BlockedActionColumn) -> Option<hygiene::BlockedAction> {
    Some(match action {
        BlockedActionColumn::SendCustomerMessage => hygiene::BlockedAction::SendCustomerMessage,
        BlockedActionColumn::MutateProviderOrPmsRecord => {
            hygiene::BlockedAction::MutateProviderOrPmsRecord
        }
        BlockedActionColumn::ChangeStaffSchedule => hygiene::BlockedAction::ChangeStaffSchedule,
        BlockedActionColumn::MoveRefundDiscountOrPayment => {
            hygiene::BlockedAction::MoveRefundDiscountOrPayment
        }
        BlockedActionColumn::HideOrAutoResolveSourceAmbiguity => {
            hygiene::BlockedAction::HideOrAutoResolveSourceAmbiguity
        }
        BlockedActionColumn::ExposeQuarantinedSensitivePayload => {
            hygiene::BlockedAction::ExposeQuarantinedSensitivePayload
        }
        BlockedActionColumn::RecordReviewedOutcome
        | BlockedActionColumn::UnauthorizedQueueWork
        | BlockedActionColumn::UnsafeSideEffect => return None,
    })
}

/// Projects blocked action into its stable storage column.
pub const fn blocked_action_column(action: hygiene::BlockedAction) -> BlockedActionColumn {
    match action {
        hygiene::BlockedAction::SendCustomerMessage => BlockedActionColumn::SendCustomerMessage,
        hygiene::BlockedAction::MutateProviderOrPmsRecord => {
            BlockedActionColumn::MutateProviderOrPmsRecord
        }
        hygiene::BlockedAction::ChangeStaffSchedule => BlockedActionColumn::ChangeStaffSchedule,
        hygiene::BlockedAction::MoveRefundDiscountOrPayment => {
            BlockedActionColumn::MoveRefundDiscountOrPayment
        }
        hygiene::BlockedAction::HideOrAutoResolveSourceAmbiguity => {
            BlockedActionColumn::HideOrAutoResolveSourceAmbiguity
        }
        hygiene::BlockedAction::ExposeQuarantinedSensitivePayload => {
            BlockedActionColumn::ExposeQuarantinedSensitivePayload
        }
    }
}
