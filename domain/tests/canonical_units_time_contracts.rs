use chrono::{NaiveDate, TimeZone, Utc};
use domain::{entities, location, money, operations, strategic_ai_ops as ops};
use uuid::Uuid;

fn location_id() -> entities::LocationId {
    entities::LocationId::new(Uuid::from_u128(0x170))
}

#[test]
fn timezone_accepts_known_iana_names_and_rejects_unknown_operational_zones() {
    let timezone = location::Timezone::try_new("  America/New_York  ").unwrap();

    assert_eq!(timezone.as_str(), "America/New_York");
    assert!(location::Timezone::try_new("Mars/Olympus_Mons").is_err());
    assert!(serde_json::from_str::<location::Timezone>(r#""America/Not_A_Zone""#).is_err());
}

#[test]
fn local_operational_windows_carry_timezone_and_validate_date_order() {
    let timezone = location::Timezone::try_new("America/Los_Angeles").unwrap();
    let start = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 8, 13).unwrap();

    let window =
        operations::operating_window::Local::new(location_id(), timezone.clone(), start, end)
            .unwrap();

    assert_eq!(window.location_id(), location_id());
    assert_eq!(window.timezone(), &timezone);
    assert_eq!(window.start_date(), start);
    assert_eq!(window.end_date(), end);
    assert!(operations::operating_window::Local::new(location_id(), timezone, end, start).is_err());
}

#[test]
fn reporting_periods_explicitly_distinguish_utc_instants_from_local_operating_dates() {
    let timezone = location::Timezone::try_new("America/New_York").unwrap();
    let local_start = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    let local_end = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
    let local = operations::reporting_period::Period::local_operating_dates(
        location_id(),
        timezone,
        local_start,
        local_end,
    )
    .unwrap();

    let utc_start = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    let utc_end = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
    let utc = operations::reporting_period::Period::utc_instants(location_id(), utc_start, utc_end)
        .unwrap();

    assert!(matches!(
        local,
        operations::reporting_period::Period::LocalOperatingDates { .. }
    ));
    assert!(matches!(
        utc,
        operations::reporting_period::Period::UtcInstants { .. }
    ));
    assert!(
        operations::reporting_period::Period::utc_instants(location_id(), utc_end, utc_start)
            .is_err()
    );
}

#[test]
fn basis_points_confidence_and_outcome_values_reject_wrong_units_and_invalid_bounds() {
    assert_eq!(money::BasisPoints::try_new(10_000).unwrap().get(), 10_000);
    assert!(money::BasisPoints::try_new(10_001).is_err());

    assert_eq!(
        ops::identity::Confidence::from_basis_points(money::BasisPoints::try_new(8_500).unwrap())
            .band(),
        ops::identity::Confidence::High
    );
    assert_eq!(
        ops::identity::Confidence::from_basis_points(money::BasisPoints::try_new(5_000).unwrap())
            .band(),
        ops::identity::Confidence::Medium
    );
    assert_eq!(
        ops::identity::Confidence::from_basis_points(money::BasisPoints::try_new(2_500).unwrap())
            .band(),
        ops::identity::Confidence::Low
    );

    let minutes = ops::outcome::MetricValue::reported_labor_minutes_difference(
        ops::labor::Minutes::try_new(15).unwrap(),
    );
    let revenue = ops::outcome::MetricValue::revenue(money::Money::usd(12_500).unwrap());
    let utilization = ops::outcome::MetricValue::utilization_basis_points(
        money::BasisPoints::try_new(125).unwrap(),
    );

    assert!(minutes.matches_metric(ops::outcome::Metric::ReportedLaborMinutesDifference));
    assert!(revenue.matches_metric(ops::outcome::Metric::ReportedRevenueObservation));
    assert!(
        utilization.matches_metric(ops::outcome::Metric::ReportedUtilizationBasisPointsObservation)
    );
    assert!(!revenue.matches_metric(ops::outcome::Metric::ReportedLaborMinutesDifference));
}

#[test]
fn capacity_recommendations_use_directional_actions_not_ambiguous_signed_minutes() {
    let add = ops::capacity::RecommendedAction::add_role_coverage(
        ops::labor::Role::FrontDesk,
        ops::labor::Minutes::try_new(60).unwrap(),
    );
    let remove = ops::capacity::RecommendedAction::remove_role_coverage(
        ops::labor::Role::FrontDesk,
        ops::labor::Minutes::try_new(30).unwrap(),
    );
    let reassign = ops::capacity::RecommendedAction::reassign_coverage(
        ops::labor::Role::KennelTechnician,
        ops::labor::Role::FrontDesk,
        ops::labor::Minutes::try_new(45).unwrap(),
    );

    assert_eq!(add.direction(), ops::capacity::CoverageDirection::Add);
    assert_eq!(remove.direction(), ops::capacity::CoverageDirection::Remove);
    assert_eq!(
        reassign.direction(),
        ops::capacity::CoverageDirection::Reassign
    );
}

#[test]
fn strategic_outcome_records_reject_metric_value_unit_mismatches() {
    let result = ops::outcome::Record::try_new(
        ops::outcome::Id::try_new("outcome-labor-42").unwrap(),
        ops::outcome::Workstream::CapacityLabor,
        location_id(),
        ops::outcome::Metric::ReportedLaborMinutesDifference,
        ops::outcome::MetricValue::revenue(money::Money::usd(25_000).unwrap()),
        ops::outcome::MetricValue::revenue(money::Money::usd(30_000).unwrap()),
        ops::outcome::Attribution::ReportedReviewedAction,
        ops::source::System::Crm,
        Utc.with_ymd_and_hms(2026, 8, 12, 16, 0, 0).unwrap(),
    );

    assert!(result.is_err());
}
