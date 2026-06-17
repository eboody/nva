use chrono::NaiveDate;
use domain::{analytics, data_quality, entities, operations, source};
use uuid::Uuid;

fn location_id() -> entities::LocationId {
    entities::LocationId(Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap())
}

#[test]
fn operating_day_key_names_the_location_service_date_join_for_future_labor_validators() {
    let key = operations::operating_day::Key::new(
        location_id(),
        operations::service_core::ServiceLine::Boarding,
        operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
            .unwrap(),
    );

    assert_eq!(key.location_id(), location_id());
    assert_eq!(
        key.service_line(),
        operations::service_core::ServiceLine::Boarding
    );
    assert_eq!(
        key.date().get(),
        NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
    );
}

#[test]
fn service_demand_fact_carries_operating_day_key_without_labor_or_capacity_schema() {
    let key = operations::operating_day::Key::new(
        location_id(),
        operations::service_core::ServiceLine::Boarding,
        operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
            .unwrap(),
    );
    let reservation_ref = source::RecordRef::new(
        source::System::Gingr,
        source::record::Id::try_new("reservation-42").unwrap(),
    );

    let fact = analytics::service_demand::Fact::try_new(
        analytics::service_demand::Id::try_new("demand-location-west-loop-boarding-2026-07-01")
            .unwrap(),
        key.clone(),
        analytics::service_demand::DemandUnits::try_new(1).unwrap(),
        vec![reservation_ref.clone()],
        analytics::ProjectionVersion::try_new("service-demand-v1").unwrap(),
        Vec::new(),
    )
    .expect("source-backed demand facts can be projected for operating-day joins");

    assert_eq!(fact.operating_day(), &key);
    assert_eq!(fact.demand_units().get(), 1);
    assert_eq!(fact.source_record_refs(), &[reservation_ref]);
    assert_eq!(fact.projection_version().as_str(), "service-demand-v1");
    assert_eq!(
        fact.data_quality_status(),
        analytics::service_demand::DataQualityStatus::Complete
    );
    assert!(fact.data_quality_issues().is_empty());
}

#[test]
fn service_demand_fact_preserves_quality_issues_without_turning_source_location_into_truth() {
    let key = operations::operating_day::Key::new(
        location_id(),
        operations::service_core::ServiceLine::Boarding,
        operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
            .unwrap(),
    );
    let reservation_ref = source::RecordRef::new(
        source::System::Gingr,
        source::record::Id::try_new("reservation-42").unwrap(),
    );
    let issue = data_quality::Issue::new(
        data_quality::Kind::AssumptionInForce {
            assumption: source::reservation::Assumption::RawPayloadRetentionUnknown,
        },
        data_quality::Severity::Warning,
        source_provenance(),
        source::Timestamp::try_new("2026-06-16T20:05:00Z").unwrap(),
        false,
    );

    let fact = analytics::service_demand::Fact::try_new(
        analytics::service_demand::Id::try_new("demand-location-west-loop-boarding-2026-07-01")
            .unwrap(),
        key,
        analytics::service_demand::DemandUnits::try_new(1).unwrap(),
        vec![reservation_ref],
        analytics::ProjectionVersion::try_new("service-demand-v1").unwrap(),
        vec![issue.clone()],
    )
    .expect("source-backed demand facts may carry review-required quality issues");

    assert_eq!(
        fact.data_quality_status(),
        analytics::service_demand::DataQualityStatus::ManagerReviewRequired
    );
    assert_eq!(fact.data_quality_issues(), &[issue]);
}

#[test]
fn service_demand_fact_rejects_empty_source_evidence() {
    let key = operations::operating_day::Key::new(
        location_id(),
        operations::service_core::ServiceLine::Boarding,
        operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
            .unwrap(),
    );

    let error = analytics::service_demand::Fact::try_new(
        analytics::service_demand::Id::try_new("demand-location-west-loop-boarding-2026-07-01")
            .unwrap(),
        key,
        analytics::service_demand::DemandUnits::try_new(1).unwrap(),
        Vec::new(),
        analytics::ProjectionVersion::try_new("service-demand-v1").unwrap(),
        Vec::new(),
    )
    .expect_err("demand facts need source evidence instead of invented counts");

    assert_eq!(
        error,
        analytics::service_demand::Error::MissingSourceEvidence
    );
}

fn source_provenance() -> source::Provenance {
    source::gingr::Provenance::builder()
        .endpoint(source::gingr::Endpoint::try_new("GET /reservations").unwrap())
        .provider_record_id(source::gingr::ProviderRecordId::try_new("reservation-42").unwrap())
        .related_provider_ids(Vec::new())
        .extraction_batch(source::gingr::ExtractionBatchId::try_new("batch-2026-06-16").unwrap())
        .pulled_at(source::Timestamp::try_new("2026-06-16T20:00:00Z").unwrap())
        .request_scope(source::gingr::RequestScope::try_new("location=west-loop").unwrap())
        .provider_schema_version(
            source::gingr::ProviderSchemaVersion::try_new("local-parser-v1").unwrap(),
        )
        .source_payload_hash(source::PayloadHash::try_new("sha256:reservation42").unwrap())
        .raw_payload_ref(
            source::RawPayloadRef::try_new("restricted://gingr/reservations/42.json").unwrap(),
        )
        .build()
        .promote()
}
