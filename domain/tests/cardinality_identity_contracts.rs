use domain::{boarding, daycare};

fn segment(
    accommodation: boarding::accommodation::Kind,
) -> boarding::capacity::NightlySegmentSnapshot {
    boarding::capacity::NightlySegmentSnapshot::from_counts(
        boarding::capacity::SegmentCounts::builder()
            .accommodation(accommodation)
            .total(boarding::capacity::RoomCount::try_new(4).unwrap())
            .occupied(boarding::capacity::RoomCount::try_new(1).unwrap())
            .build(),
    )
}

#[test]
fn recurring_attendance_weekdays_are_a_non_empty_duplicate_free_set() {
    let error =
        daycare::attendance::Days::try_new(vec![chrono::Weekday::Mon, chrono::Weekday::Mon])
            .expect_err("the same weekday twice is one recurrence day, not two");

    assert_eq!(
        error,
        daycare::attendance::DaysError::Duplicate {
            day: chrono::Weekday::Mon,
        }
    );
}

#[test]
fn capacity_snapshot_has_one_authoritative_segment_per_accommodation() {
    let error = boarding::capacity::Snapshot::new(vec![
        segment(boarding::accommodation::Kind::LuxuryDogSuite),
        segment(boarding::accommodation::Kind::LuxuryDogSuite),
    ])
    .expect_err("duplicate room segments make capacity authority ambiguous");

    assert_eq!(
        error,
        boarding::capacity::SnapshotError::DuplicateAccommodation {
            accommodation: boarding::accommodation::Kind::LuxuryDogSuite,
        }
    );
}

#[test]
fn accommodation_alternatives_are_non_empty_unique_and_ordered_by_preference() {
    assert_eq!(
        boarding::accommodation::Alternatives::try_new(Vec::new()),
        Err(boarding::accommodation::AlternativesError::Empty)
    );
    assert_eq!(
        boarding::accommodation::Alternatives::try_new(vec![
            boarding::accommodation::Kind::LuxuryDogSuite,
            boarding::accommodation::Kind::LuxuryDogSuite,
        ]),
        Err(boarding::accommodation::AlternativesError::Duplicate {
            accommodation: boarding::accommodation::Kind::LuxuryDogSuite,
        })
    );

    let alternatives = boarding::accommodation::Alternatives::try_new(vec![
        boarding::accommodation::Kind::LuxuryDogSuite,
        boarding::accommodation::Kind::ClassicDogSuite,
    ])
    .unwrap();
    let preference = boarding::accommodation::Preference::AnyOf(alternatives);

    assert_eq!(
        preference.acceptable_kinds(),
        vec![
            boarding::accommodation::Kind::LuxuryDogSuite,
            boarding::accommodation::Kind::ClassicDogSuite,
        ]
    );
}
