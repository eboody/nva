use chrono::{TimeZone, Utc};
use domain::{entities, incident, policy};
use uuid::Uuid;

fn main() {
    let incident_id = entities::IncidentId::new(uuid::Uuid::from_u128(1));
    let incident = entities::Incident::builder()
        .id(incident_id)
        .location_id(entities::LocationId::new(Uuid::from_u128(2)))
        .primary_subject(entities::IncidentSubject::Customer(entities::CustomerId::new(
            Uuid::from_u128(3),
        )))
        .category(incident::Category::CustomerService)
        .severity(incident::Severity::High)
        .status(incident::Status::NeedsManagerReview)
        .reported_by(entities::ActorRef::System)
        .reported_at(Utc.with_ymd_and_hms(2026, 8, 15, 8, 0, 0).unwrap())
        .summary(incident::Summary::try_new("review required").unwrap())
        .required_review_gates(vec![policy::ReviewGate::ManagerApproval])
        .build()
        .unwrap();
    let approval = entities::approval::Record::builder()
        .id(entities::approval::Id::new(Uuid::from_u128(4)))
        .target(entities::approval::Target::Incident(incident_id))
        .gate(policy::ReviewGate::ManagerApproval)
        .lifecycle(entities::approval::Lifecycle::Approved {
            decided_by: entities::ActorRef::System,
            decided_at: Utc.with_ymd_and_hms(2026, 8, 15, 10, 0, 0).unwrap(),
        })
        .requested_by(entities::ActorRef::System)
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 15, 9, 0, 0).unwrap())
        .build()
        .unwrap();

    let _ = incident.close_with(approval);
}
