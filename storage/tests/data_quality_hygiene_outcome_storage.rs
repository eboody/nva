use storage::operations::{
    DataQualityHygieneActionKindCode, DataQualityHygieneLineageIds,
    DataQualityHygieneLocalPersistenceRecords, DataQualityHygieneOutcomeCode,
    DataQualityHygieneOutcomeRecord, DataQualityHygieneOutcomeSummary,
    DataQualityHygienePersonaCode, DataQualityResolutionStatusCode, ReviewGateCode,
    ReviewPacketStatusCode, StoredDataQualityHygieneLaborMinutes, StoredSourceRecordRef,
    WorkflowResultStatusCode,
};
use strum::VariantArray;

#[test]
fn data_quality_hygiene_outcome_record_codecs_preserve_labor_and_provenance() {
    let record = DataQualityHygieneOutcomeRecord::builder()
        .action_id("dq-action-dq-missing-vaccine-42".to_owned())
        .outcome(DataQualityHygieneOutcomeCode::Completed)
        .before_minutes(StoredDataQualityHygieneLaborMinutes::try_new(25).unwrap())
        .actual_minutes(StoredDataQualityHygieneLaborMinutes::try_new(9).unwrap())
        .actor_id("front-desk-lead-1".to_owned())
        .actor_persona(DataQualityHygienePersonaCode::FrontDeskLead)
        .feedback(
            "Found the source document; PMS correction remains outside this workflow.".to_owned(),
        )
        .source_refs(vec![source_ref()])
        .issue_refs(vec!["dq-missing-vaccine-42".to_owned()])
        .resolution_status_after_review(DataQualityResolutionStatusCode::Acknowledged)
        .recorded_at("2026-06-17T14:30:00Z".to_owned())
        .correlation_id("data-quality-hygiene:location-1:2026-06-17".to_owned())
        .location_id("location-1".to_owned())
        .operating_day("2026-06-17".to_owned())
        .action_kind(DataQualityHygieneActionKindCode::ReviewStaleVaccinationSourceFreshness)
        .owner_persona(DataQualityHygienePersonaCode::FrontDeskLead)
        .reported_estimated_minutes_difference(15)
        .build();
    let decoded =
        DataQualityHygieneOutcomeRecord::decode_json(&record.encode_json().unwrap()).unwrap();
    assert_eq!(decoded, record);
    assert_eq!(decoded.source_refs.len(), 1);
    assert_eq!(decoded.issue_refs, ["dq-missing-vaccine-42"]);
    let debug = format!("{decoded:?}");
    assert!(!debug.contains("front-desk-lead-1"));
    assert!(!debug.contains("Found the source document"));
    assert!(!debug.contains("pet-vaccine-42"));
    assert!(debug.contains("[REDACTED]"));
}

#[test]
fn data_quality_hygiene_outcome_decoder_requires_supported_schema_version() {
    let mut payload = serde_json::to_value(outcome_record(
        "dq-action-versioned",
        DataQualityHygieneOutcomeCode::Completed,
        25,
        9,
        "dq-versioned",
        "pet-versioned",
    ))
    .unwrap();
    payload.as_object_mut().unwrap().remove("schema_version");
    assert!(DataQualityHygieneOutcomeRecord::decode_json(&payload.to_string()).is_err());

    payload["schema_version"] = serde_json::json!("data_quality_hygiene_outcome.v99");
    assert!(DataQualityHygieneOutcomeRecord::decode_json(&payload.to_string()).is_err());
}

#[test]
fn stored_data_quality_hygiene_minutes_reject_zero_values() {
    let error = StoredDataQualityHygieneLaborMinutes::try_new(0).unwrap_err();
    assert!(error.to_string().contains("must be greater than zero"));
}

#[test]
fn data_quality_hygiene_storage_codes_roundtrip_through_strum_variant_metadata() {
    for outcome in DataQualityHygieneOutcomeCode::VARIANTS {
        assert_eq!(outcome.to_string().parse(), Ok(*outcome));
    }

    for persona in DataQualityHygienePersonaCode::VARIANTS {
        assert_eq!(persona.to_string().parse(), Ok(*persona));
    }

    for action_kind in DataQualityHygieneActionKindCode::VARIANTS {
        assert_eq!(action_kind.to_string().parse(), Ok(*action_kind));
    }

    for resolution_status in DataQualityResolutionStatusCode::VARIANTS {
        assert_eq!(
            resolution_status.to_string().parse(),
            Ok(*resolution_status)
        );
    }
}

#[test]
fn data_quality_hygiene_outcome_summary_aggregates_reviewed_labor_loop_proof() {
    let completed = outcome_record(
        "dq-action-dq-missing-vaccine-42",
        DataQualityHygieneOutcomeCode::Completed,
        25,
        9,
        "dq-missing-vaccine-42",
        "pet-vaccine-42",
    );
    let wrong_source = outcome_record(
        "dq-action-dq-duplicate-customer-17",
        DataQualityHygieneOutcomeCode::SourceFactWasWrong,
        30,
        12,
        "dq-duplicate-customer-17",
        "customer-17",
    );

    let summary = DataQualityHygieneOutcomeSummary::from_records(
        &[completed, wrong_source],
        "location-1",
        "2026-06-17",
        Some("data-quality-hygiene:location-1:2026-06-17"),
    );

    assert_eq!(summary.location_id, "location-1");
    assert_eq!(summary.operating_day, "2026-06-17");
    assert_eq!(
        summary.correlation_id.as_deref(),
        Some("data-quality-hygiene:location-1:2026-06-17")
    );
    assert_eq!(summary.reviewed_outcome_count, 2);
    assert_eq!(summary.reported_completed_outcome_count, 1);
    assert_eq!(summary.deferred_count, 0);
    assert_eq!(summary.wrong_source_count, 1);
    assert_eq!(summary.not_actionable_count, 0);
    assert_eq!(summary.total_reported_estimated_minutes_difference, 0);
    assert_eq!(summary.total_actual_minutes_spent, 21);
    assert_eq!(summary.source_refs.len(), 2);
    assert_eq!(
        summary.issue_refs,
        ["dq-duplicate-customer-17", "dq-missing-vaccine-42"]
    );
}

#[test]
fn staff_completed_data_quality_outcome_does_not_manufacture_manager_approval_authority() {
    let records = DataQualityHygieneLocalPersistenceRecords::from_reviewed_outcome(
        DataQualityHygieneLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000101".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000102".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000103".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000104".to_owned())
            .subject_id("00000000-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("dqh:location-1:2026-06-17:context".to_owned())
            .recorded_at("2026-06-17T14:30:00Z".to_owned())
            .build(),
        outcome_record(
            "dq-action-dq-missing-vaccine-42",
            DataQualityHygieneOutcomeCode::Completed,
            25,
            9,
            "dq-missing-vaccine-42",
            "pet-vaccine-42",
        ),
    );

    assert_eq!(records.workflow_event.workflow_name, "data-quality-hygiene");
    assert_eq!(records.workflow_event.event_kind, "context_created");
    assert_eq!(records.workflow_event.subject_kind, "location");
    assert_eq!(
        records.workflow_event.payload["live_side_effects_allowed"],
        false
    );
    assert_eq!(
        records.workflow_result.status,
        WorkflowResultStatusCode::Succeeded
    );
    assert_eq!(records.review_packet.gate, ReviewGateCode::ManagerApproval);
    assert_eq!(
        records.review_packet.status,
        ReviewPacketStatusCode::ReadyForReview
    );
    assert_eq!(records.approval_record.status, "approval_requested");
    assert_eq!(records.approval_record.decided_by_actor_kind, None);
    assert_eq!(records.approval_record.decided_by_actor_id, None);
    assert_eq!(records.approval_record.decided_at, None);
    assert_eq!(records.outcome.workflow_event_id, records.workflow_event.id);
    assert_eq!(
        records.outcome.approval_record_id,
        records.approval_record.id
    );
    assert_eq!(records.audit_events.len(), 2);

    assert!(records.outbox_candidate.is_none());
}

#[test]
fn data_quality_hygiene_lineage_does_not_create_outbox_for_deferred_outcomes() {
    let records = DataQualityHygieneLocalPersistenceRecords::from_reviewed_outcome(
        DataQualityHygieneLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000201".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000202".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000203".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000204".to_owned())
            .subject_id("00000000-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("dqh:location-1:2026-06-17:deferred".to_owned())
            .recorded_at("2026-06-17T14:30:00Z".to_owned())
            .build(),
        outcome_record(
            "dq-action-dq-missing-vaccine-43",
            DataQualityHygieneOutcomeCode::Deferred,
            25,
            9,
            "dq-missing-vaccine-43",
            "pet-vaccine-43",
        ),
    );

    assert_eq!(
        records.workflow_result.status,
        WorkflowResultStatusCode::NeedsReview
    );
    assert!(records.outbox_candidate.is_none());
}

#[test]
fn data_quality_hygiene_lineage_records_rejection_evidence_without_outbox_for_wrong_source() {
    let records = DataQualityHygieneLocalPersistenceRecords::from_reviewed_outcome(
        DataQualityHygieneLineageIds::builder()
            .workflow_event_id("00000000-0000-0000-0000-000000000301".to_owned())
            .review_packet_id("00000000-0000-0000-0000-000000000302".to_owned())
            .approval_record_id("00000000-0000-0000-0000-000000000303".to_owned())
            .outbox_record_id("00000000-0000-0000-0000-000000000304".to_owned())
            .subject_id("00000000-0000-0000-0000-000000000001".to_owned())
            .idempotency_key("dqh:location-1:2026-06-17:wrong-source".to_owned())
            .recorded_at("2026-06-17T14:30:00Z".to_owned())
            .build(),
        outcome_record(
            "dq-action-dq-missing-vaccine-44",
            DataQualityHygieneOutcomeCode::SourceFactWasWrong,
            25,
            9,
            "dq-missing-vaccine-44",
            "pet-vaccine-44",
        ),
    );

    assert_eq!(
        records.workflow_result.status,
        WorkflowResultStatusCode::NeedsReview
    );
    assert_eq!(
        records.review_packet.status,
        ReviewPacketStatusCode::ReadyForReview
    );
    assert_eq!(records.approval_record.status, "approval_requested");
    assert_eq!(records.approval_record.decided_by_actor_kind, None);
    assert_eq!(records.approval_record.decided_by_actor_id, None);
    assert_eq!(records.approval_record.decided_at, None);
    assert!(records.outbox_candidate.is_none());
}

fn source_ref() -> StoredSourceRecordRef {
    StoredSourceRecordRef::builder()
        .system("gingr".to_owned())
        .record_type("pet_vaccination".to_owned())
        .record_id("pet-vaccine-42".to_owned())
        .observed_at("2026-06-17T00:00:00Z".to_owned())
        .adapter_version("gingr-v0-readonly".to_owned())
        .build()
}

fn outcome_record(
    action_id: &str,
    outcome: DataQualityHygieneOutcomeCode,
    before_minutes: u16,
    actual_minutes: u16,
    issue_ref: &str,
    source_record_id: &str,
) -> DataQualityHygieneOutcomeRecord {
    DataQualityHygieneOutcomeRecord::builder()
        .action_id(action_id.to_owned())
        .outcome(outcome)
        .before_minutes(StoredDataQualityHygieneLaborMinutes::try_new(before_minutes).unwrap())
        .actual_minutes(StoredDataQualityHygieneLaborMinutes::try_new(actual_minutes).unwrap())
        .actor_id("front-desk-lead-1".to_owned())
        .actor_persona(DataQualityHygienePersonaCode::FrontDeskLead)
        .feedback("Reviewed source-grounded hygiene issue without provider writes.".to_owned())
        .source_refs(vec![
            StoredSourceRecordRef::builder()
                .system("gingr".to_owned())
                .record_type("source_record".to_owned())
                .record_id(source_record_id.to_owned())
                .observed_at("2026-06-17T00:00:00Z".to_owned())
                .adapter_version("gingr-v0-readonly".to_owned())
                .build(),
        ])
        .issue_refs(vec![issue_ref.to_owned()])
        .resolution_status_after_review(DataQualityResolutionStatusCode::Acknowledged)
        .recorded_at("2026-06-17T14:30:00Z".to_owned())
        .correlation_id("data-quality-hygiene:location-1:2026-06-17".to_owned())
        .location_id("location-1".to_owned())
        .operating_day("2026-06-17".to_owned())
        .action_kind(DataQualityHygieneActionKindCode::ReviewStaleVaccinationSourceFreshness)
        .owner_persona(DataQualityHygienePersonaCode::FrontDeskLead)
        .reported_estimated_minutes_difference(before_minutes.saturating_sub(actual_minutes))
        .build()
}
