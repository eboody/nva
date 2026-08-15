use chrono::{TimeZone, Utc};
use domain::{analytics, boarding, daycare, entities, location, operations, retail, training};

fn location_id() -> entities::LocationId {
    entities::LocationId::new(uuid::Uuid::from_u128(1))
}

fn operating_day() -> operations::operating_day::Key {
    operations::operating_day::Key::new(
        location_id(),
        operations::service_core::ServiceLine::Boarding,
        operations::operating_day::Date::try_new(
            chrono::NaiveDate::from_ymd_opt(2026, 8, 13).unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn operating_period_serde_rejects_every_interval_rejected_by_constructors() {
    let timezone = location::Timezone::try_new("America/Los_Angeles").unwrap();
    let start_date = chrono::NaiveDate::from_ymd_opt(2026, 8, 13).unwrap();
    let end_date = chrono::NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();

    assert!(
        operations::operating_window::Local::new(
            location_id(),
            timezone.clone(),
            start_date,
            end_date,
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<operations::operating_window::Local>(serde_json::json!({
            "location_id": location_id(),
            "timezone": timezone,
            "start_date": start_date,
            "end_date": end_date,
        }))
        .is_err(),
        "Serde must use the local operating-window constructor invariant"
    );

    let start = Utc.with_ymd_and_hms(2026, 8, 13, 12, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 8, 13, 11, 0, 0).unwrap();
    assert!(operations::reporting_period::Period::utc_instants(location_id(), start, end).is_err());
    assert!(
        serde_json::from_value::<operations::reporting_period::Period>(serde_json::json!({
            "UtcInstants": {
                "location_id": location_id(),
                "start": start,
                "end": end,
            }
        }))
        .is_err(),
        "Serde must use the UTC reporting-period constructor invariant"
    );
}

#[test]
fn training_serde_rejects_unsupported_or_empty_evidence_values() {
    let unsupported_claim = serde_json::json!({
        "outcome": "CanineGoodCitizenReadiness",
        "status": "Achieved",
        "evidence": [],
        "milestones": []
    });
    assert!(
        serde_json::from_value::<training::outcome::Claim>(unsupported_claim).is_err(),
        "Serde must use Claim::from_evidence"
    );
    assert!(
        serde_json::from_value::<training::outcome::Claims>(serde_json::json!([])).is_err(),
        "Serde must preserve the non-empty outcome-claim invariant"
    );
    assert!(
        serde_json::from_value::<training::progress::EvidenceSet>(serde_json::json!([])).is_err(),
        "Serde must preserve the non-empty progress-evidence invariant"
    );
}

#[test]
fn equivalent_domain_value_serde_rejects_constructor_invalid_values() {
    assert!(
        serde_json::from_value::<daycare::attendance::DateRange>(serde_json::json!({
            "start": "2026-08-13",
            "end": "2026-08-12"
        }))
        .is_err()
    );
    assert!(serde_json::from_value::<daycare::attendance::Days>(serde_json::json!([])).is_err());
    assert!(
        serde_json::from_value::<boarding::ServiceWindow>(serde_json::json!({
            "start": 18,
            "end": 8
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<boarding::capacity::Snapshot>(serde_json::json!({
            "segments": []
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<retail::inventory::Position>(serde_json::json!({
            "location_id": location_id(),
            "sku": "food-001",
            "on_hand": 2,
            "reserved": 3,
            "reorder_at": 1
        }))
        .is_err(),
        "Serde must use Position::record"
    );
    assert!(
        serde_json::from_value::<analytics::ProjectionVersion>(serde_json::json!(" ")).is_err()
    );
    assert!(serde_json::from_value::<analytics::stay::Id>(serde_json::json!("")).is_err());
    assert!(
        serde_json::from_value::<analytics::service_demand::Id>(serde_json::json!(" ")).is_err()
    );

    let demand = analytics::service_demand::Fact::try_new(
        analytics::service_demand::Id::try_new("demand-1").unwrap(),
        operating_day(),
        analytics::service_demand::DemandUnits::try_new(1).unwrap(),
        vec![domain::source::RecordRef::new(
            domain::source::System::Gingr,
            domain::source::record::Id::try_new("reservation-1").unwrap(),
        )],
        analytics::ProjectionVersion::try_new("demand-v1").unwrap(),
        Vec::new(),
    )
    .unwrap();
    let mut demand_json = serde_json::to_value(demand).unwrap();
    demand_json["source_record_refs"] = serde_json::json!([]);
    assert!(
        serde_json::from_value::<analytics::service_demand::Fact>(demand_json).is_err(),
        "Serde must preserve required source evidence for promoted demand facts"
    );
}
