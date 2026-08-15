use domain::{boarding, entities, policy};

fn count(value: u16) -> boarding::capacity::RoomCount {
    boarding::capacity::RoomCount::try_new(value).unwrap()
}

fn segment(total: u16, occupied: u16) -> boarding::capacity::NightlySegmentSnapshot {
    boarding::capacity::NightlySegmentSnapshot::from_counts(
        boarding::capacity::SegmentCounts::builder()
            .accommodation(boarding::accommodation::Kind::LuxuryDogSuite)
            .total(count(total))
            .occupied(count(occupied))
            .build(),
    )
}

fn request() -> boarding::capacity::Request {
    boarding::capacity::Request::new(
        entities::LocationId::new(uuid::Uuid::from_u128(1)),
        entities::Species::Dog,
        boarding::accommodation::Preference::Specific(
            boarding::accommodation::Kind::LuxuryDogSuite,
        ),
    )
}

fn segment_for(
    accommodation: boarding::accommodation::Kind,
    total: u16,
    occupied: u16,
) -> boarding::capacity::NightlySegmentSnapshot {
    boarding::capacity::NightlySegmentSnapshot::from_counts(
        boarding::capacity::SegmentCounts::builder()
            .accommodation(accommodation)
            .total(count(total))
            .occupied(count(occupied))
            .build(),
    )
}

#[test]
fn occupied_above_total_is_typed_reconciliation_evidence() {
    let nightly = segment(10, 12);
    let anomaly = boarding::capacity::OverOccupancy::try_new(count(10), count(12)).unwrap();

    assert_eq!(
        nightly.occupancy_state(),
        boarding::capacity::OccupancyState::OverOccupied(anomaly)
    );
    assert_eq!(anomaly.total(), count(10));
    assert_eq!(anomaly.occupied(), count(12));
    assert_eq!(anomaly.excess(), count(2));
}

#[test]
fn over_occupancy_evidence_rejects_non_contradictory_counts_and_invalid_json() {
    assert_eq!(
        boarding::capacity::OverOccupancy::try_new(count(10), count(10)),
        Err(boarding::capacity::OverOccupancyError::NotOverOccupied {
            total: count(10),
            occupied: count(10),
        })
    );

    let invalid = r#"{"total":10,"occupied":12,"excess":1}"#;
    assert!(serde_json::from_str::<boarding::capacity::OverOccupancy>(invalid).is_err());
}

#[test]
fn over_occupancy_evidence_round_trips_through_validated_serde() {
    let anomaly = boarding::capacity::OverOccupancy::try_new(count(10), count(12)).unwrap();
    let json = serde_json::to_string(&anomaly).unwrap();

    assert_eq!(
        serde_json::from_str::<boarding::capacity::OverOccupancy>(&json).unwrap(),
        anomaly
    );
}

#[test]
fn policy_requires_reconciliation_instead_of_waitlisting_or_confirming_over_occupancy() {
    let snapshot = boarding::capacity::Snapshot::new(vec![segment(10, 12)]).unwrap();
    let anomaly = boarding::capacity::OverOccupancy::try_new(count(10), count(12)).unwrap();

    let decision = boarding::capacity::Policy.evaluate(&request(), &snapshot);

    assert_eq!(
        decision,
        boarding::capacity::Decision::ReconciliationRequired {
            anomaly,
            review_gate: policy::ReviewGate::ManagerApproval,
        }
    );
    assert_eq!(
        decision.required_review_gate(),
        Some(policy::ReviewGate::ManagerApproval)
    );
}

#[test]
fn policy_does_not_confirm_an_alternative_while_eligible_inventory_is_contradictory() {
    let luxury = boarding::accommodation::Kind::LuxuryDogSuite;
    let classic = boarding::accommodation::Kind::ClassicDogSuite;
    let snapshot = boarding::capacity::Snapshot::new(vec![
        segment_for(luxury, 10, 9),
        segment_for(classic, 10, 12),
    ])
    .unwrap();
    let preference = boarding::accommodation::Preference::AnyOf(
        boarding::accommodation::Alternatives::try_new(vec![luxury, classic]).unwrap(),
    );
    let request = boarding::capacity::Request::new(
        entities::LocationId::new(uuid::Uuid::from_u128(1)),
        entities::Species::Dog,
        preference,
    );

    assert!(matches!(
        boarding::capacity::Policy.evaluate(&request, &snapshot),
        boarding::capacity::Decision::ReconciliationRequired { .. }
    ));
}
