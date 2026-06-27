use chrono::NaiveDate;
use uuid::Uuid;

use app::data_quality_hygiene;
use app::data_quality_hygiene::AuthorizationPolicy;
use domain::{data_quality, entities, operations, policy, source};

#[test]
fn data_quality_hygiene_context_builds_source_grounded_internal_actions_with_labor_delta() {
    let request = data_quality_hygiene::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(data_quality_hygiene::HygienePersona::GeneralManager)
        .candidates(vec![candidate(
            "dq-missing-vaccine-42",
            data_quality_hygiene::CandidateKind::SourceIssue,
            data_quality::Kind::MissingVaccinationRecord,
        )])
        .build();

    let packet = data_quality_hygiene::Workflow::evaluate(request);

    assert_eq!(packet.workflow(), data_quality_hygiene::WORKFLOW_NAME);
    assert_eq!(packet.actions().len(), 1);
    assert!(packet.all_actions_are_source_grounded());
    assert_eq!(packet.before_minutes().get(), 25);
    assert_eq!(packet.after_minutes().get(), 10);
    assert_eq!(packet.minutes_saved(), 15);
    assert!(
        packet
            .safe_agent_actions()
            .contains(&data_quality_hygiene::SafeAgentAction::PreserveAmbiguityForReview)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::HideOrAutoResolveSourceAmbiguity)
    );

    let action = &packet.actions()[0];
    assert_eq!(
        action.kind(),
        data_quality_hygiene::ActionKind::ReviewStaleVaccinationSourceFreshness
    );
    assert_eq!(
        action.owner_persona(),
        data_quality_hygiene::HygienePersona::FrontDeskLead
    );
    assert_eq!(
        action.removed_manual_work(),
        data_quality_hygiene::RemovedManualWork::SourceFreshnessReview
    );
    assert_eq!(
        action.required_review_gates(),
        &[policy::ReviewGate::ManagerApproval]
    );
    assert_eq!(action.issue_refs(), &[issue_ref("dq-missing-vaccine-42")]);
}

#[test]
fn data_quality_hygiene_draft_validation_rejects_ambiguity_hiding_and_side_effects_fail_closed() {
    let packet = data_quality_hygiene::Workflow::evaluate(
        data_quality_hygiene::Request::builder()
            .location_id(location_id())
            .operating_day(operating_day())
            .prepared_for(data_quality_hygiene::HygienePersona::GeneralManager)
            .candidates(vec![candidate(
                "dq-service-name-77",
                data_quality_hygiene::CandidateKind::ServiceLineMapping,
                data_quality::Kind::UnmappedServiceType,
            )])
            .build(),
    );
    let action = packet.actions()[0].clone();

    let accepted = data_quality_hygiene::DraftSubmission::builder()
        .context_packet_id(packet.context_packet_id().clone())
        .correlation_id(packet.correlation_id().clone())
        .actions(vec![data_quality_hygiene::DraftAction::from_action(
            action.clone(),
        )])
        .build();
    assert!(packet.validate_draft(&accepted).is_accepted());

    let hidden = data_quality_hygiene::DraftSubmission::builder()
        .context_packet_id(packet.context_packet_id().clone())
        .correlation_id(packet.correlation_id().clone())
        .actions(vec![
            data_quality_hygiene::DraftAction::from_action(action.clone())
                .with_attempted_ambiguity_resolution(),
        ])
        .build();
    assert_eq!(
        packet.validate_draft(&hidden).rejection_reasons(),
        &[data_quality_hygiene::DraftRejectionReason::AttemptedAmbiguityHiding]
    );

    let known_blocked = data_quality_hygiene::DraftSubmission::builder()
        .context_packet_id(packet.context_packet_id().clone())
        .correlation_id(packet.correlation_id().clone())
        .actions(vec![
            data_quality_hygiene::DraftAction::from_action(action.clone())
                .with_requested_side_effect("send_customer_message"),
        ])
        .build();
    assert_eq!(
        packet.validate_draft(&known_blocked).rejection_reasons(),
        &[data_quality_hygiene::DraftRejectionReason::BlockedSideEffectRequested]
    );

    let unknown_side_effect = data_quality_hygiene::DraftSubmission::builder()
        .context_packet_id(packet.context_packet_id().clone())
        .correlation_id(packet.correlation_id().clone())
        .actions(vec![
            data_quality_hygiene::DraftAction::from_action(action)
                .with_requested_side_effect("repair_source_record_directly"),
        ])
        .build();
    assert_eq!(
        packet
            .validate_draft(&unknown_side_effect)
            .rejection_reasons(),
        &[data_quality_hygiene::DraftRejectionReason::UnsupportedSideEffectRequested]
    );
}

#[test]
fn data_quality_hygiene_value_objects_reject_blank_deserialization() {
    assert!(serde_json::from_str::<data_quality_hygiene::IssueRef>(r#""   ""#).is_err());
    assert!(serde_json::from_str::<data_quality_hygiene::ActionId>(r#""   ""#).is_err());
    assert!(serde_json::from_str::<data_quality_hygiene::ContextPacketId>(r#""   ""#).is_err());
    assert!(serde_json::from_str::<data_quality_hygiene::CorrelationId>(r#""   ""#).is_err());
    assert!(serde_json::from_str::<data_quality_hygiene::ActionRationale>(r#""   ""#).is_err());
}

#[test]
fn data_quality_hygiene_value_objects_deserialize_with_constructor_hygiene() {
    let issue_ref: data_quality_hygiene::IssueRef =
        serde_json::from_str(r#""  dq-issue-7  ""#).unwrap();
    let action_id: data_quality_hygiene::ActionId =
        serde_json::from_str(r#""  dq-action-7  ""#).unwrap();
    let context_packet_id: data_quality_hygiene::ContextPacketId =
        serde_json::from_str(r#""  dq-context-7  ""#).unwrap();
    let correlation_id: data_quality_hygiene::CorrelationId =
        serde_json::from_str(r#""  dq-correlation-7  ""#).unwrap();
    let action_rationale: data_quality_hygiene::ActionRationale =
        serde_json::from_str(r#""  Review stale vaccine source evidence before cleanup  ""#)
            .unwrap();

    assert_eq!(issue_ref.as_str(), "dq-issue-7");
    assert_eq!(action_id.as_str(), "dq-action-7");
    assert_eq!(context_packet_id.as_str(), "dq-context-7");
    assert_eq!(correlation_id.as_str(), "dq-correlation-7");
    assert_eq!(
        serde_json::to_value(&action_rationale).unwrap(),
        serde_json::json!("Review stale vaccine source evidence before cleanup")
    );
}

#[test]
fn data_quality_hygiene_outcome_records_reject_empty_source_or_issue_proof() {
    let without_source_refs = data_quality_hygiene::OutcomeRecord::builder()
        .action_id(data_quality_hygiene::ActionId::try_new("dq-action-1").unwrap())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(data_quality_hygiene::FeedbackOutcome::Completed)
        .before_minutes(data_quality_hygiene::LaborMinutes::try_new(25).unwrap())
        .actual_minutes(data_quality_hygiene::LaborMinutes::try_new(9).unwrap())
        .issue_refs(vec![issue_ref("dq-missing-vaccine-42")])
        .reviewed_resolution_status(data_quality::ResolutionStatus::Acknowledged)
        .build();
    assert_eq!(
        without_source_refs,
        Err(data_quality_hygiene::Error::OutcomeSourceRecordRefRequired)
    );

    let without_issue_refs = data_quality_hygiene::OutcomeRecord::builder()
        .action_id(data_quality_hygiene::ActionId::try_new("dq-action-1").unwrap())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(data_quality_hygiene::FeedbackOutcome::Completed)
        .before_minutes(data_quality_hygiene::LaborMinutes::try_new(25).unwrap())
        .actual_minutes(data_quality_hygiene::LaborMinutes::try_new(9).unwrap())
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &source_provenance(),
        )])
        .reviewed_resolution_status(data_quality::ResolutionStatus::Acknowledged)
        .build();
    assert_eq!(
        without_issue_refs,
        Err(data_quality_hygiene::Error::OutcomeIssueRefRequired)
    );
}

#[test]
fn data_quality_hygiene_candidate_names_affected_entity_issue_category_and_redaction_boundary() {
    let vaccine_candidate = candidate(
        "dq-missing-vaccine-42",
        data_quality_hygiene::CandidateKind::SourceIssue,
        data_quality::Kind::MissingVaccinationRecord,
    );
    assert_eq!(
        vaccine_candidate.affected_entity(),
        data_quality_hygiene::AffectedEntity::VaccinationRecord
    );
    assert_eq!(
        vaccine_candidate.issue_category(),
        data_quality_hygiene::IssueCategory::SourceFreshness
    );
    assert_eq!(
        vaccine_candidate.redaction_policy(),
        data_quality_hygiene::RedactionPolicy::SummarizeMetadataOnly
    );
    assert!(
        !vaccine_candidate
            .redaction_policy()
            .allows_agent_payload_access()
    );

    let payment_candidate = candidate(
        "dq-payment-conflict-42",
        data_quality_hygiene::CandidateKind::SourceIssue,
        data_quality::Kind::PaymentStateConflict,
    );
    assert_eq!(
        payment_candidate.affected_entity(),
        data_quality_hygiene::AffectedEntity::PaymentRecord
    );
    assert_eq!(
        payment_candidate.issue_category(),
        data_quality_hygiene::IssueCategory::PaymentConflict
    );
    assert_eq!(
        payment_candidate.redaction_policy(),
        data_quality_hygiene::RedactionPolicy::QuarantineRawPayload
    );
    assert!(
        !payment_candidate
            .redaction_policy()
            .allows_agent_payload_access()
    );
}

#[test]
fn data_quality_hygiene_actions_name_cleanup_action_and_reviewer_role_without_provider_write() {
    let packet = data_quality_hygiene::Workflow::evaluate(
        data_quality_hygiene::Request::builder()
            .location_id(location_id())
            .operating_day(operating_day())
            .prepared_for(data_quality_hygiene::HygienePersona::GeneralManager)
            .candidates(vec![candidate(
                "dq-duplicate-pet-42",
                data_quality_hygiene::CandidateKind::DuplicateCandidate,
                data_quality::Kind::DuplicateSourceRecord,
            )])
            .build(),
    );

    let action = &packet.actions()[0];
    assert_eq!(
        action.cleanup_action(),
        data_quality_hygiene::CleanupAction::PrepareDuplicateReview
    );
    assert_eq!(
        action.reviewer_role(),
        data_quality_hygiene::ReviewerRole::GeneralManager
    );
    assert!(action.requires_human_or_system_of_record_review());
    assert!(
        packet
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord)
    );
}

#[test]
fn data_quality_hygiene_payment_conflicts_prepare_protected_payment_review_without_money_movement()
{
    let packet = data_quality_hygiene::Workflow::evaluate(
        data_quality_hygiene::Request::builder()
            .location_id(location_id())
            .operating_day(operating_day())
            .prepared_for(data_quality_hygiene::HygienePersona::GeneralManager)
            .candidates(vec![candidate(
                "dq-payment-conflict-42",
                data_quality_hygiene::CandidateKind::SourceIssue,
                data_quality::Kind::PaymentStateConflict,
            )])
            .build(),
    );

    let action = &packet.actions()[0];
    assert_eq!(
        action.cleanup_action(),
        data_quality_hygiene::CleanupAction::PreparePaymentConflictReview
    );
    assert_eq!(
        action.reviewer_role(),
        data_quality_hygiene::ReviewerRole::FrontDeskLead
    );
    assert!(action.requires_human_or_system_of_record_review());
    assert!(
        action
            .required_review_gates()
            .contains(&policy::ReviewGate::RefundOrDepositException)
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord)
    );
}

#[test]
fn data_quality_hygiene_derived_action_id_preserves_max_length_issue_ref_without_panicking() {
    let max_length_issue_ref = format!("dq-{}", "x".repeat(117));
    assert_eq!(max_length_issue_ref.len(), 120);

    let packet = data_quality_hygiene::Workflow::evaluate(
        data_quality_hygiene::Request::builder()
            .location_id(location_id())
            .operating_day(operating_day())
            .prepared_for(data_quality_hygiene::HygienePersona::GeneralManager)
            .candidates(vec![candidate(
                &max_length_issue_ref,
                data_quality_hygiene::CandidateKind::SourceIssue,
                data_quality::Kind::MissingVaccinationRecord,
            )])
            .build(),
    );

    let action = &packet.actions()[0];
    assert_eq!(action.issue_refs(), &[issue_ref(&max_length_issue_ref)]);
    assert_eq!(action.id().as_str().len(), "dq-action-".len() + 120);
}

#[test]
fn data_quality_hygiene_non_completed_outcomes_keep_feedback_but_do_not_claim_labor_savings() {
    for outcome in [
        data_quality_hygiene::FeedbackOutcome::Deferred,
        data_quality_hygiene::FeedbackOutcome::SuppressedByManager,
        data_quality_hygiene::FeedbackOutcome::SourceFactWasWrong,
        data_quality_hygiene::FeedbackOutcome::NotActionable,
    ] {
        let record = outcome_record(outcome);
        assert!(!record.outcome().can_claim_labor_savings());
        assert!(!record.labor_minutes_are_claimable());
        assert_eq!(record.actual_minutes_saved(), 16);
        assert!(!record.source_record_refs().is_empty());
        assert!(!record.issue_refs().is_empty());
    }
}

#[test]
fn data_quality_hygiene_outcome_records_actual_minutes_without_external_mutation() {
    let outcome = data_quality_hygiene::OutcomeRecord::builder()
        .action_id(data_quality_hygiene::ActionId::try_new("dq-action-1").unwrap())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(data_quality_hygiene::FeedbackOutcome::Completed)
        .before_minutes(data_quality_hygiene::LaborMinutes::try_new(25).unwrap())
        .actual_minutes(data_quality_hygiene::LaborMinutes::try_new(9).unwrap())
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &source_provenance(),
        )])
        .issue_refs(vec![issue_ref("dq-missing-vaccine-42")])
        .reviewed_resolution_status(data_quality::ResolutionStatus::Acknowledged)
        .build()
        .unwrap();

    assert_eq!(outcome.actual_minutes_saved(), 16);
    assert_eq!(
        outcome.reviewed_resolution_status(),
        Some(data_quality::ResolutionStatus::Acknowledged)
    );
    assert!(outcome.records_feedback_without_external_mutation());
    assert!(
        outcome
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert!(
        outcome
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::SendCustomerMessage)
    );
}

#[test]
fn manager_actor_records_reviewed_outcome_through_app_owned_ports() {
    let action_id = data_quality_hygiene::ActionId::try_new("dq-action-1").unwrap();
    let mut service = fixture_outcome_capture_service();

    let receipt = service
        .record_reviewed_outcome(data_quality_hygiene::OutcomeCaptureRequest::new(
            data_quality_hygiene::ActorId::try_new("gm-riley").unwrap(),
            outcome_record(data_quality_hygiene::FeedbackOutcome::Completed),
        ))
        .unwrap();

    assert_eq!(receipt.action_id(), &action_id);
    assert_eq!(service.outcome_recorder().outcomes().len(), 1);
    assert_eq!(service.audit_log().records().len(), 1);
    assert_eq!(service.blocked_action_log().records().len(), 0);
    assert_eq!(
        service.outcome_recorder().outcomes()[0].action_id(),
        &action_id
    );
}

#[test]
fn staff_actor_without_manager_gate_records_blocked_action_instead_of_outcome() {
    let mut service = fixture_outcome_capture_service();
    service
        .actor_directory_mut()
        .add_actor(data_quality_hygiene::ActorAssignment::new(
            data_quality_hygiene::ActorId::try_new("front-desk-1").unwrap(),
            entities::ActorRef::Staff {
                staff_id: entities::StaffId::try_new("front-desk-1").unwrap(),
            },
            data_quality_hygiene::ReviewerRole::FrontDeskAgent,
            vec![location_id()],
        ));

    let result = service.record_reviewed_outcome(data_quality_hygiene::OutcomeCaptureRequest::new(
        data_quality_hygiene::ActorId::try_new("front-desk-1").unwrap(),
        outcome_record(data_quality_hygiene::FeedbackOutcome::Completed),
    ));

    assert_eq!(result, Err(data_quality_hygiene::Error::ActorNotAuthorized));
    assert_eq!(service.outcome_recorder().outcomes().len(), 0);
    assert_eq!(service.audit_log().records().len(), 0);
    assert_eq!(service.blocked_action_log().records().len(), 1);
    assert_eq!(
        service.blocked_action_log().records()[0].reason(),
        data_quality_hygiene::BlockedActionReason::ActorLacksReviewGate
    );
}

#[test]
fn role_location_authorization_separates_queue_work_from_manager_outcomes() {
    let policy = data_quality_hygiene::RoleLocationAuthorization;
    let review_item = data_quality_hygiene::ReviewQueueItem::new(
        data_quality_hygiene::ActionId::try_new("dq-action-1").unwrap(),
        location_id(),
        vec![policy::ReviewGate::ManagerApproval],
    );
    let alice = data_quality_hygiene::ActorAssignment::new(
        data_quality_hygiene::ActorId::try_new("alice").unwrap(),
        entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("staff-alice").unwrap(),
        },
        data_quality_hygiene::ReviewerRole::FrontDeskLead,
        vec![location_id()],
    );
    let sam = data_quality_hygiene::ActorAssignment::new(
        data_quality_hygiene::ActorId::try_new("sam").unwrap(),
        entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new("staff-sam").unwrap(),
        },
        data_quality_hygiene::ReviewerRole::FrontDeskLead,
        vec![location_id_202()],
    );
    let morgan = data_quality_hygiene::ActorAssignment::new(
        data_quality_hygiene::ActorId::try_new("morgan").unwrap(),
        entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("manager-morgan").unwrap(),
        },
        data_quality_hygiene::ReviewerRole::GeneralManager,
        vec![location_id()],
    );
    let ai = data_quality_hygiene::ActorAssignment::new(
        data_quality_hygiene::ActorId::try_new("dq-ai").unwrap(),
        entities::ActorRef::System,
        data_quality_hygiene::ReviewerRole::OperationsAnalyst,
        vec![location_id()],
    );

    assert!(policy.can_work_queue_item(&alice, &review_item));
    assert!(!policy.can_work_queue_item(&sam, &review_item));
    assert!(!policy.can_record_outcome(&alice, &review_item));
    assert!(policy.can_record_outcome(&morgan, &review_item));
    assert!(policy.can_work_queue_item(&ai, &review_item));
    assert!(!policy.can_record_outcome(&ai, &review_item));
}

fn fixture_outcome_capture_service() -> data_quality_hygiene::OutcomeCaptureService<
    FixtureActorDirectory,
    data_quality_hygiene::RoleLocationAuthorization,
    FixtureReviewQueueStore,
    FixtureOutcomeRecorder,
    FixtureAuditLog,
    FixtureBlockedActionLog,
> {
    let action_id = data_quality_hygiene::ActionId::try_new("dq-action-1").unwrap();
    data_quality_hygiene::OutcomeCaptureService::new(
        FixtureActorDirectory::new(vec![data_quality_hygiene::ActorAssignment::new(
            data_quality_hygiene::ActorId::try_new("gm-riley").unwrap(),
            entities::ActorRef::Manager {
                manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
            },
            data_quality_hygiene::ReviewerRole::GeneralManager,
            vec![location_id()],
        )]),
        data_quality_hygiene::RoleLocationAuthorization,
        FixtureReviewQueueStore::new(vec![data_quality_hygiene::ReviewQueueItem::new(
            action_id,
            location_id(),
            vec![policy::ReviewGate::ManagerApproval],
        )]),
        FixtureOutcomeRecorder::default(),
        FixtureAuditLog::default(),
        FixtureBlockedActionLog::default(),
    )
}

#[derive(Debug, Default)]
struct FixtureActorDirectory {
    actors: Vec<data_quality_hygiene::ActorAssignment>,
}

impl FixtureActorDirectory {
    fn new(actors: Vec<data_quality_hygiene::ActorAssignment>) -> Self {
        Self { actors }
    }

    fn add_actor(&mut self, actor: data_quality_hygiene::ActorAssignment) {
        self.actors.push(actor);
    }
}

impl data_quality_hygiene::ActorDirectory for FixtureActorDirectory {
    fn resolve_actor(
        &self,
        actor_id: &data_quality_hygiene::ActorId,
    ) -> Option<data_quality_hygiene::ActorAssignment> {
        self.actors
            .iter()
            .find(|actor| actor.actor_id() == actor_id)
            .cloned()
    }
}

#[derive(Debug)]
struct FixtureReviewQueueStore {
    review_items: Vec<data_quality_hygiene::ReviewQueueItem>,
}

impl FixtureReviewQueueStore {
    fn new(review_items: Vec<data_quality_hygiene::ReviewQueueItem>) -> Self {
        Self { review_items }
    }
}

impl data_quality_hygiene::ReviewQueueStore for FixtureReviewQueueStore {
    fn review_item_for_action(
        &self,
        action_id: &data_quality_hygiene::ActionId,
    ) -> Option<data_quality_hygiene::ReviewQueueItem> {
        self.review_items
            .iter()
            .find(|item| item.action_id() == action_id)
            .cloned()
    }
}

#[derive(Debug, Default)]
struct FixtureOutcomeRecorder {
    outcomes: Vec<data_quality_hygiene::OutcomeRecord>,
}

impl FixtureOutcomeRecorder {
    fn outcomes(&self) -> &[data_quality_hygiene::OutcomeRecord] {
        &self.outcomes
    }
}

impl data_quality_hygiene::OutcomeRecorder for FixtureOutcomeRecorder {
    fn record_outcome(
        &mut self,
        outcome: data_quality_hygiene::OutcomeRecord,
    ) -> data_quality_hygiene::OutcomeReceipt {
        let receipt = data_quality_hygiene::OutcomeReceipt::new(outcome.action_id().clone());
        self.outcomes.push(outcome);
        receipt
    }
}

#[derive(Debug, Default)]
struct FixtureAuditLog {
    records: Vec<data_quality_hygiene::AuditRecord>,
}

impl FixtureAuditLog {
    fn records(&self) -> &[data_quality_hygiene::AuditRecord] {
        &self.records
    }
}

impl data_quality_hygiene::AuditLog for FixtureAuditLog {
    fn append_audit_record(&mut self, record: data_quality_hygiene::AuditRecord) {
        self.records.push(record);
    }
}

#[derive(Debug, Default)]
struct FixtureBlockedActionLog {
    records: Vec<data_quality_hygiene::BlockedActionRecord>,
}

impl FixtureBlockedActionLog {
    fn records(&self) -> &[data_quality_hygiene::BlockedActionRecord] {
        &self.records
    }
}

impl data_quality_hygiene::BlockedActionLog for FixtureBlockedActionLog {
    fn record_blocked_action(&mut self, record: data_quality_hygiene::BlockedActionRecord) {
        self.records.push(record);
    }
}

fn candidate(
    id: &str,
    kind: data_quality_hygiene::CandidateKind,
    issue_kind: data_quality::Kind,
) -> data_quality_hygiene::Candidate {
    data_quality_hygiene::Candidate::builder()
        .id(issue_ref(id))
        .kind(kind)
        .issue(data_quality::Issue::new(
            issue_kind,
            data_quality::Severity::Warning,
            source_provenance(),
            source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap(),
            false,
        ))
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &source_provenance(),
        )])
        .source_freshness(data_quality_hygiene::SourceFreshness::Stale)
        .sensitivity(data_quality_hygiene::Sensitivity::VaccineEvidence)
        .build()
}

fn issue_ref(id: &str) -> data_quality_hygiene::IssueRef {
    data_quality_hygiene::IssueRef::try_new(id).unwrap()
}

fn outcome_record(
    outcome: data_quality_hygiene::FeedbackOutcome,
) -> data_quality_hygiene::OutcomeRecord {
    data_quality_hygiene::OutcomeRecord::builder()
        .action_id(data_quality_hygiene::ActionId::try_new("dq-action-1").unwrap())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-riley").unwrap(),
        })
        .outcome(outcome)
        .before_minutes(data_quality_hygiene::LaborMinutes::try_new(25).unwrap())
        .actual_minutes(data_quality_hygiene::LaborMinutes::try_new(9).unwrap())
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &source_provenance(),
        )])
        .issue_refs(vec![issue_ref("dq-missing-vaccine-42")])
        .reviewed_resolution_status(data_quality::ResolutionStatus::Acknowledged)
        .build()
        .unwrap()
}

fn source_provenance() -> source::Provenance {
    source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /pets/{id}/vaccinations").unwrap())
        .record_id(source::record::Id::try_new("pet-vaccine-42").unwrap())
        .extraction_batch(source::ExtractionBatchId::try_new("dq-hygiene-batch-local").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z").unwrap())
        .request_scope(
            source::RequestScope::try_new("local-data-quality-hygiene-contract").unwrap(),
        )
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly").unwrap())
        .payload_hash(source::PayloadHash::try_new("sha256:dqhygienefixture").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("fixtures/gingr/data-quality-hygiene.json").unwrap(),
        )
        .build()
}

fn location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0001))
}

fn location_id_202() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0002))
}

fn operating_day() -> operations::operating_day::Date {
    operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 6, 17).unwrap()).unwrap()
}
