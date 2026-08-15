use chrono::{TimeZone, Utc};
use domain::{entities, policy};
use uuid::Uuid;

fn main() {
    let message_id = entities::MessageId::new(uuid::Uuid::from_u128(1));
    let approval = entities::approval::Record::builder()
        .id(entities::approval::Id::new(Uuid::from_u128(2)))
        .target(entities::approval::Target::Message(message_id))
        .gate(policy::ReviewGate::CustomerMessageApproval)
        .lifecycle(entities::approval::Lifecycle::Approved {
            decided_by: entities::ActorRef::System,
            decided_at: Utc.with_ymd_and_hms(2026, 8, 15, 10, 0, 0).unwrap(),
        })
        .requested_by(entities::ActorRef::System)
        .requested_at(Utc.with_ymd_and_hms(2026, 8, 15, 9, 0, 0).unwrap())
        .build()
        .unwrap();

    let reviewer_authority = entities::message_record::ReviewerAuthority {
        approval_id: approval.id(),
        message_id,
        gate: policy::ReviewGate::CustomerMessageApproval,
        reviewer: entities::ActorRef::System,
    };
    let _ = entities::message_record::authorize_queue(&approval, reviewer_authority);
}
