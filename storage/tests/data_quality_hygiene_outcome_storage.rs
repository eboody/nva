use storage::operations::{
    DataQualityHygieneActionKindCode, DataQualityHygieneOutcomeCode,
    DataQualityHygieneOutcomeRecord, DataQualityHygieneOutcomeSummary,
    DataQualityHygienePersonaCode, DataQualityResolutionStatusCode,
    StoredDataQualityHygieneLaborMinutes, StoredSourceRecordRef,
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
        .estimated_minutes_saved(15)
        .build();

    assert_eq!(record.actual_minutes_saved(), 16);
    let decoded =
        DataQualityHygieneOutcomeRecord::decode_json(&record.encode_json().unwrap()).unwrap();
    assert_eq!(decoded, record);
    assert_eq!(decoded.source_refs.len(), 1);
    assert_eq!(decoded.issue_refs, ["dq-missing-vaccine-42"]);
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
    assert_eq!(summary.completed_count, 1);
    assert_eq!(summary.deferred_count, 0);
    assert_eq!(summary.wrong_source_count, 1);
    assert_eq!(summary.not_actionable_count, 0);
    assert_eq!(summary.total_estimated_minutes_saved, 34);
    assert_eq!(summary.total_actual_minutes_spent, 21);
    assert_eq!(summary.completed_actual_minutes_saved, 16);
    assert_eq!(summary.source_refs.len(), 2);
    assert_eq!(
        summary.issue_refs,
        ["dq-duplicate-customer-17", "dq-missing-vaccine-42"]
    );
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
        .estimated_minutes_saved(before_minutes.saturating_sub(actual_minutes))
        .build()
}
