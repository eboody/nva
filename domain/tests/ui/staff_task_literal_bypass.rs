use chrono::{TimeZone, Utc};
use domain::{entities, staff, workflow};
use uuid::Uuid;

fn main() {
    let _completed_without_evidence = staff::Task {
        location_id: entities::LocationId(Uuid::from_u128(1)),
        kind: staff::task::Kind::MedicationAdministration {
            pet_id: entities::PetId(Uuid::from_u128(2)),
        },
        title: workflow::task::Title::try_new("Give evening medication").unwrap(),
        status: staff::task::Status::Completed,
        priority: staff::task::Priority::High,
        due_at: Utc.with_ymd_and_hms(2026, 8, 12, 9, 0, 0).unwrap(),
        assignment: staff::task::Assignment::Role(staff::Role::KennelTechnician),
        source: staff::task::Source::Reservation(entities::reservation::Id(Uuid::from_u128(3))),
        completion_evidence: None,
    };
}
