use chrono::NaiveDate;
use domain::{analytics, operations, source};

#[test]
fn operating_day_key_names_the_location_service_date_join_for_future_labor_validators() {
    let key = operations::operating_day::Key::new(
        source::record::Id::try_new("location-west-loop").unwrap(),
        operations::service_core::ServiceLine::Boarding,
        operations::operating_day::Date::try_new(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
            .unwrap(),
    );

    assert_eq!(key.location_record_id().as_str(), "location-west-loop");
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
        source::record::Id::try_new("location-west-loop").unwrap(),
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
    )
    .expect("source-backed demand facts can be projected for operating-day joins");

    assert_eq!(fact.operating_day(), &key);
    assert_eq!(fact.demand_units().get(), 1);
    assert_eq!(fact.source_record_refs(), &[reservation_ref]);
    assert_eq!(fact.projection_version().as_str(), "service-demand-v1");
}

#[test]
fn service_demand_fact_rejects_empty_source_evidence() {
    let key = operations::operating_day::Key::new(
        source::record::Id::try_new("location-west-loop").unwrap(),
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
    )
    .expect_err("demand facts need source evidence instead of invented counts");

    assert_eq!(
        error,
        analytics::service_demand::Error::MissingSourceEvidence
    );
}
