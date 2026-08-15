use domain::entities;

fn main() {
    let _ = entities::message_record::QueueAuthorization {
        message_id: entities::MessageId::new(uuid::Uuid::from_u128(1)),
        evidence: todo!(),
    };
}
