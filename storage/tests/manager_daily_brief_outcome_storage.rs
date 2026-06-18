#[test]
fn manager_daily_brief_outcome_records_roundtrip_labor_savings_evidence() {
    let record = storage::operations::ManagerDailyBriefOutcomeRecord::builder()
        .action_id("checkout-exception-reservation-4242".to_owned())
        .outcome(storage::operations::ManagerDailyBriefOutcomeCode::Completed)
        .before_minutes(
            storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(20).unwrap(),
        )
        .actual_minutes(
            storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(12).unwrap(),
        )
        .actor_id("front-desk-lead-17".to_owned())
        .actor_persona(storage::operations::ManagerDailyBriefPersonaCode::FrontDeskLead)
        .feedback("Resolved before checkout rush; brief saved a manual open-stay audit.".to_owned())
        .source_refs(vec![
            storage::operations::StoredSourceRecordRef::builder()
                .system("gingr".to_owned())
                .record_type("reservation".to_owned())
                .record_id("reservation-4242".to_owned())
                .observed_at("2026-06-17T12:00:00Z".to_owned())
                .adapter_version("local-manager-daily-brief-outcome-fixture-v1".to_owned())
                .build(),
        ])
        .recorded_at("2026-06-17T13:15:00Z".to_owned())
        .correlation_id(
            "manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17".to_owned(),
        )
        .location_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
        .operating_day("2026-06-17".to_owned())
        .action_kind(storage::operations::ManagerDailyBriefActionKindCode::ResolveCheckoutException)
        .owner_persona(storage::operations::ManagerDailyBriefPersonaCode::FrontDeskLead)
        .estimated_minutes_saved(12)
        .build();

    let encoded = record.encode_json().unwrap();
    let decoded =
        storage::operations::ManagerDailyBriefOutcomeRecord::decode_json(&encoded).unwrap();

    assert_eq!(decoded, record);
    assert_eq!(decoded.actual_minutes_saved(), 8);
    assert_eq!(
        decoded.reporting_group().location_id,
        "00c0ffee-0000-0000-0000-000000000001"
    );
    assert_eq!(decoded.reporting_group().operating_day, "2026-06-17");
    assert_eq!(
        decoded.reporting_group().action_kind,
        storage::operations::ManagerDailyBriefActionKindCode::ResolveCheckoutException
    );
    assert_eq!(
        decoded.reporting_group().owner_persona,
        storage::operations::ManagerDailyBriefPersonaCode::FrontDeskLead
    );
}

#[test]
fn manager_daily_brief_outcome_records_reject_zero_labor_minutes_at_storage_boundary() {
    let raw = r#"{
        "action_id":"checkout-exception-reservation-4242",
        "outcome":"completed",
        "before_minutes":20,
        "actual_minutes":0,
        "actor_id":"front-desk-lead-17",
        "actor_persona":"front_desk_lead",
        "feedback":"Resolved before checkout rush.",
        "source_refs":[],
        "recorded_at":"2026-06-17T13:15:00Z",
        "correlation_id":"manager-daily-brief:00c0ffee-0000-0000-0000-000000000001:2026-06-17",
        "location_id":"00c0ffee-0000-0000-0000-000000000001",
        "operating_day":"2026-06-17",
        "action_kind":"resolve_checkout_exception",
        "owner_persona":"front_desk_lead",
        "estimated_minutes_saved":12
    }"#;

    assert!(storage::operations::ManagerDailyBriefOutcomeRecord::decode_json(raw).is_err());
}
