use strum::VariantArray;

#[test]
fn crm_retention_outcome_records_roundtrip_recovered_booking_and_no_action_correlation() {
    let recovered = crm_retention_record(
        "crm-retention:reservation-42:accepted",
        storage::operations::CrmRetentionOutcomeCode::RecoveredBooking,
        storage::operations::CrmRetentionReviewedOutcomeClassificationCode::RecoveredBooking,
        "crm-segment-retention-42",
    );
    let no_action = crm_retention_record(
        "crm-retention:reservation-43:wrong-source",
        storage::operations::CrmRetentionOutcomeCode::WrongSource,
        storage::operations::CrmRetentionReviewedOutcomeClassificationCode::NoActionOutcome,
        "crm-segment-retention-43",
    );

    let decoded = storage::operations::CrmRetentionOutcomeRecord::decode_json(
        &recovered.encode_json().unwrap(),
    )
    .unwrap();
    assert_eq!(decoded, recovered);
    assert_eq!(decoded.source_refs.len(), 1);
    assert_eq!(decoded.recommendation_id, "retention-action-reservation-42");
    assert_eq!(decoded.review_packet_id, "retention-review-packet-42");

    let summary = storage::operations::CrmRetentionOutcomeSummary::from_records(
        &[recovered, no_action],
        "00c0ffee-0000-0000-0000-000000000001",
        "2026-06-17",
        None,
    );

    assert_eq!(summary.reviewed_outcome_count, 2);
    assert_eq!(summary.recovered_booking_count, 1);
    assert_eq!(summary.no_action_outcome_count, 1);
    assert_eq!(summary.source_refs.len(), 2);
    assert_eq!(
        summary.recommendation_ids,
        [
            "retention-action-reservation-42",
            "retention-action-reservation-43"
        ]
    );
}

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
    let serialized: serde_json::Value = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, record);
    assert_eq!(
        serialized["schema_version"],
        "manager_daily_brief_outcome.v0"
    );
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
fn manager_daily_brief_capacity_labor_outcome_records_roundtrip_reviewed_recommendation_correlation()
 {
    let record = storage::operations::ManagerDailyBriefOutcomeRecord::builder()
        .action_id("capacity-labor-boarding-front-desk".to_owned())
        .outcome(storage::operations::ManagerDailyBriefOutcomeCode::Completed)
        .before_minutes(
            storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(50).unwrap(),
        )
        .actual_minutes(
            storage::operations::StoredManagerDailyBriefLaborMinutes::try_new(18).unwrap(),
        )
        .actor_id("gm-capacity-review".to_owned())
        .actor_persona(storage::operations::ManagerDailyBriefPersonaCode::GeneralManager)
        .feedback("Reviewed capacity/labor evidence and adjusted the internal staffing plan manually; no agent schedule mutation occurred.".to_owned())
        .source_refs(vec![
            storage::operations::StoredSourceRecordRef::builder()
                .system("business_intelligence".to_owned())
                .record_type("service_capacity_forecast".to_owned())
                .record_id("capacity-demand-boarding-2026-06-17".to_owned())
                .observed_at("2026-06-17T06:00:00Z".to_owned())
                .adapter_version("capacity-labor-v1".to_owned())
                .build(),
            storage::operations::StoredSourceRecordRef::builder()
                .system("labor_scheduling".to_owned())
                .record_type("scheduled_coverage".to_owned())
                .record_id("labor-coverage-front-desk-2026-06-17".to_owned())
                .observed_at("2026-06-17T06:00:00Z".to_owned())
                .adapter_version("capacity-labor-v1".to_owned())
                .build(),
        ])
        .recorded_at("2026-06-17T13:15:00Z".to_owned())
        .correlation_id(
            "manager-daily-brief:capacity-labor:00c0ffee-0000-0000-0000-000000000001:2026-06-17".to_owned(),
        )
        .location_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
        .operating_day("2026-06-17".to_owned())
        .action_kind(storage::operations::ManagerDailyBriefActionKindCode::ReviewCapacityLaborRecommendation)
        .owner_persona(storage::operations::ManagerDailyBriefPersonaCode::GeneralManager)
        .estimated_minutes_saved(32)
        .build();

    let decoded = storage::operations::ManagerDailyBriefOutcomeRecord::decode_json(
        &record.encode_json().unwrap(),
    )
    .unwrap();

    assert_eq!(decoded, record);
    assert_eq!(decoded.actual_minutes_saved(), 32);
    assert_eq!(decoded.source_refs.len(), 2);
    assert_eq!(
        decoded.reporting_group().action_kind,
        storage::operations::ManagerDailyBriefActionKindCode::ReviewCapacityLaborRecommendation
    );
}

#[test]
fn site_finance_outcome_records_roundtrip_review_audit_and_claimability_links() {
    let record = storage::operations::SiteFinanceOutcomeRecord::builder()
        .correlation_id("site-finance:00c0ffee:2026-06".to_owned())
        .location_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
        .period_start("2026-06-01T00:00:00Z".to_owned())
        .period_end("2026-07-01T00:00:00Z".to_owned())
        .service("boarding".to_owned())
        .recommendation_id("site-finance-review:00c0ffee:2026-06".to_owned())
        .review_packet_id("site-finance-review:00c0ffee:2026-06".to_owned())
        .audit_event_id("audit:site-finance-review:00c0ffee:2026-06".to_owned())
        .legal_action("record_reviewed_recommendation_only".to_owned())
        .value_attribution(storage::operations::SiteFinanceValueAttribution::ReviewedAction)
        .workflow_completion(storage::operations::SiteFinanceWorkflowCompletion::Completed)
        .manager_approval(
            storage::operations::SiteFinanceManagerApproval::approved_by_manager(
                "general-manager-1".to_owned(),
                "2026-07-02T12:00:00Z".to_owned(),
                None,
                "site-finance-approval:00c0ffee:2026-06".to_owned(),
                "message".to_owned(),
                "00c0ffee-0000-0000-0000-000000000001".to_owned(),
            ),
        )
        .can_support_value_claim(true)
        .currency("usd".to_owned())
        .net_revenue_minor_units(159_000)
        .variance_minor_units(9_000)
        .source_refs(vec![
            storage::operations::StoredSourceRecordRef::builder()
                .system("finance_accounting".to_owned())
                .record_type("site_period_finance".to_owned())
                .record_id("site-finance-boarding-2026-06-current".to_owned())
                .observed_at("2026-07-01T12:00:00Z".to_owned())
                .adapter_version("site-finance-fixture-v1".to_owned())
                .build(),
        ])
        .recorded_at("2026-07-02T12:00:00Z".to_owned())
        .build();

    let decoded =
        storage::operations::SiteFinanceOutcomeRecord::decode_json(&record.encode_json().unwrap())
            .unwrap();

    assert_eq!(decoded, record);
    assert_eq!(
        decoded.review_packet_id,
        "site-finance-review:00c0ffee:2026-06"
    );
    assert_eq!(
        decoded.audit_event_id,
        "audit:site-finance-review:00c0ffee:2026-06"
    );
    assert_eq!(decoded.legal_action, "record_reviewed_recommendation_only");
    assert!(decoded.can_support_value_claim);
    assert_eq!(decoded.source_refs.len(), 1);
}

#[test]
fn manager_daily_brief_storage_codes_roundtrip_through_strum_variant_metadata() {
    for outcome in storage::operations::ManagerDailyBriefOutcomeCode::VARIANTS {
        assert_eq!(outcome.to_string().parse(), Ok(*outcome));
    }

    for persona in storage::operations::ManagerDailyBriefPersonaCode::VARIANTS {
        assert_eq!(persona.to_string().parse(), Ok(*persona));
    }

    for action_kind in storage::operations::ManagerDailyBriefActionKindCode::VARIANTS {
        assert_eq!(action_kind.to_string().parse(), Ok(*action_kind));
    }
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

fn crm_retention_record(
    correlation_id: &str,
    outcome: storage::operations::CrmRetentionOutcomeCode,
    classification: storage::operations::CrmRetentionReviewedOutcomeClassificationCode,
    source_record_id: &str,
) -> storage::operations::CrmRetentionOutcomeRecord {
    storage::operations::CrmRetentionOutcomeRecord::builder()
        .correlation_id(correlation_id.to_owned())
        .recommendation_id(
            correlation_id
                .replace("crm-retention:", "retention-action-")
                .replace(":accepted", "")
                .replace(":wrong-source", ""),
        )
        .review_packet_id(
            correlation_id
                .replace("crm-retention:reservation-", "retention-review-packet-")
                .replace(":accepted", "")
                .replace(":wrong-source", ""),
        )
        .outcome(outcome)
        .reviewed_outcome_classification(classification)
        .actor_id("front-desk-lead-17".to_owned())
        .actor_persona(storage::operations::ManagerDailyBriefPersonaCode::FrontDeskLead)
        .feedback(
            "Reviewed CRM retention outcome without live sends, bookings, or provider writes."
                .to_owned(),
        )
        .source_refs(vec![
            storage::operations::StoredSourceRecordRef::builder()
                .system("crm".to_owned())
                .record_type("retention_segment".to_owned())
                .record_id(source_record_id.to_owned())
                .observed_at("2026-06-17T12:00:00Z".to_owned())
                .adapter_version("crm-v0-readonly".to_owned())
                .build(),
        ])
        .recorded_at("2026-06-17T13:15:00Z".to_owned())
        .location_id("00c0ffee-0000-0000-0000-000000000001".to_owned())
        .operating_day("2026-06-17".to_owned())
        .build()
}
