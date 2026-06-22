use chrono::NaiveDate;
use uuid::Uuid;

use app::data_quality_hygiene;
use domain::{data_quality, entities, operations, source};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = data_quality_hygiene::Request::builder()
        .location_id(location_id())
        .operating_day(operating_day())
        .prepared_for(data_quality_hygiene::HygienePersona::GeneralManager)
        .candidates(vec![candidate(
            "dq-missing-vaccine-local-smoke",
            data_quality_hygiene::CandidateKind::SourceIssue,
            data_quality::Kind::MissingVaccinationRecord,
        )?])
        .build();

    let packet = data_quality_hygiene::Workflow::evaluate(request);
    assert_eq!(packet.workflow(), data_quality_hygiene::WORKFLOW_NAME);
    assert!(packet.all_actions_are_source_grounded());
    assert!(packet.minutes_saved() > 0);
    assert!(
        packet
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::SendCustomerMessage),
        "customer sends must remain blocked in the local smoke"
    );
    assert!(
        packet
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord),
        "provider/PMS writes must remain blocked in the local smoke"
    );

    println!(
        "context_ok workflow={} actions={} estimated_minutes_saved={} live_side_effects_allowed=false",
        packet.workflow(),
        packet.actions().len(),
        packet.minutes_saved()
    );

    let action = packet
        .actions()
        .first()
        .expect("local fixture creates one data-quality hygiene action")
        .clone();
    let accepted_draft = data_quality_hygiene::DraftSubmission::builder()
        .context_packet_id(packet.context_packet_id().clone())
        .correlation_id(packet.correlation_id().clone())
        .actions(vec![data_quality_hygiene::DraftAction::from_action(
            action.clone(),
        )])
        .build();
    let validation = packet.validate_draft(&accepted_draft);
    assert!(
        validation.is_accepted(),
        "expected source-grounded draft to pass"
    );
    println!("draft_validation_ok accepted_actions=1 requested_side_effects=0");

    let blocked_draft = data_quality_hygiene::DraftSubmission::builder()
        .context_packet_id(packet.context_packet_id().clone())
        .correlation_id(packet.correlation_id().clone())
        .actions(vec![
            data_quality_hygiene::DraftAction::from_action(action.clone())
                .with_requested_side_effect("send_customer_message"),
        ])
        .build();
    let blocked_validation = packet.validate_draft(&blocked_draft);
    assert_eq!(
        blocked_validation.rejection_reasons(),
        &[data_quality_hygiene::DraftRejectionReason::BlockedSideEffectRequested]
    );
    println!("blocked_draft_validation_ok blocked_side_effect=send_customer_message");

    let outcome = data_quality_hygiene::OutcomeRecord::builder()
        .action_id(action.id().clone())
        .recorded_by(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new("gm-local-smoke")?,
        })
        .outcome(data_quality_hygiene::FeedbackOutcome::Completed)
        .before_minutes(action.labor_impact().before_minutes())
        .actual_minutes(data_quality_hygiene::LaborMinutes::try_new(8)?)
        .source_record_refs(action.source_record_refs().to_vec())
        .issue_refs(action.issue_refs().to_vec())
        .reviewed_resolution_status(data_quality::ResolutionStatus::Acknowledged)
        .build()?;

    assert!(outcome.records_feedback_without_external_mutation());
    assert!(outcome.actual_minutes_saved() > 0);
    assert!(
        outcome
            .blocked_actions()
            .contains(&data_quality_hygiene::BlockedAction::MutateProviderOrPmsRecord),
        "outcome capture must not authorize provider/PMS mutation"
    );
    println!(
        "outcome_ok estimated_minutes_saved={} actual_minutes_saved={} live_side_effects_allowed=false",
        packet.minutes_saved(),
        outcome.actual_minutes_saved()
    );

    Ok(())
}

fn candidate(
    id: &str,
    kind: data_quality_hygiene::CandidateKind,
    issue_kind: data_quality::Kind,
) -> Result<data_quality_hygiene::Candidate, Box<dyn std::error::Error>> {
    Ok(data_quality_hygiene::Candidate::builder()
        .id(data_quality_hygiene::IssueRef::try_new(id)?)
        .kind(kind)
        .issue(data_quality::Issue::new(
            issue_kind,
            data_quality::Severity::Warning,
            source_provenance()?,
            source::Timestamp::try_new("2026-06-17T00:00:00Z")?,
            false,
        ))
        .source_record_refs(vec![source::RecordRef::from_provenance(
            &source_provenance()?,
        )])
        .source_freshness(data_quality_hygiene::SourceFreshness::Stale)
        .sensitivity(data_quality_hygiene::Sensitivity::VaccineEvidence)
        .build())
}

fn source_provenance() -> Result<source::Provenance, Box<dyn std::error::Error>> {
    Ok(source::Provenance::builder()
        .system(source::System::Gingr)
        .endpoint(source::Endpoint::try_new("GET /pets/{id}/vaccinations")?)
        .record_id(source::record::Id::try_new("pet-vaccine-local-smoke")?)
        .extraction_batch(source::ExtractionBatchId::try_new(
            "dq-hygiene-local-smoke-batch",
        )?)
        .pulled_at(source::Timestamp::try_new("2026-06-17T00:00:00Z")?)
        .request_scope(source::RequestScope::try_new(
            "local-data-quality-hygiene-smoke-readonly-fixture",
        )?)
        .schema_version(source::SchemaVersion::try_new("gingr-v0-readonly-fixture")?)
        .payload_hash(source::PayloadHash::try_new("sha256:dqhygienelocalsmoke")?)
        .raw_payload_ref(source::RawPayloadRef::try_new(
            "fixtures/gingr/data-quality-hygiene-local-smoke.json",
        )?)
        .build())
}

fn location_id() -> entities::LocationId {
    entities::LocationId(Uuid::from_u128(0x00c0_ffee_0000_0000_0000_0000_0000_0001))
}

fn operating_day() -> operations::operating_day::Date {
    operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 6, 17).unwrap()).unwrap()
}
