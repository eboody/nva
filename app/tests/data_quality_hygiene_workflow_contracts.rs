use chrono::NaiveDate;
use uuid::Uuid;

use app::data_quality_hygiene;
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
        .build();

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

fn operating_day() -> operations::operating_day::Date {
    operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 6, 17).unwrap()).unwrap()
}
