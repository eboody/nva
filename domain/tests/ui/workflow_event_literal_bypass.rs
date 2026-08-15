use chrono::{TimeZone, Utc};
use domain::{entities, policy, workflow};
use uuid::Uuid;

fn main() {
    let _workflow_event_bypass = workflow::Event {
        event_id: workflow::EventId::new(Uuid::from_u128(35)),
        event_type: workflow::EventType::CheckoutCompleted,
        occurred_at: Utc.with_ymd_and_hms(2026, 8, 12, 9, 0, 0).unwrap(),
        actor: entities::ActorRef::System,
        location_id: entities::LocationId::new(Uuid::from_u128(36)),
        subject: workflow::Subject::Customer(entities::CustomerId::new(Uuid::from_u128(37))),
        policy_context: workflow::PolicyContext {
            allowed_actions: vec![workflow::AllowedAction::ReadEntities],
            automation_level: policy::automation::Level::DraftOnly,
            required_reviews: vec![policy::ReviewGate::ManagerApproval],
        },
    };
}
