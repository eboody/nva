use storage::operations::{
    DataQualityHygieneActionKindCode, DataQualityHygieneOutcomeCode,
    DataQualityHygieneOutcomeRecord, DataQualityHygienePersonaCode,
    DataQualityResolutionStatusCode, StoredDataQualityHygieneLaborMinutes, StoredSourceRecordRef,
};

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

fn source_ref() -> StoredSourceRecordRef {
    StoredSourceRecordRef::builder()
        .system("gingr".to_owned())
        .record_type("pet_vaccination".to_owned())
        .record_id("pet-vaccine-42".to_owned())
        .observed_at("2026-06-17T00:00:00Z".to_owned())
        .adapter_version("gingr-v0-readonly".to_owned())
        .build()
}
