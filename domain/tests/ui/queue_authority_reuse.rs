use domain::{entities, message, policy};
use uuid::Uuid;

fn reuse(
    authority: entities::message_record::QueueAuthorization,
    first: entities::Message,
    second: entities::Message,
) {
    let _ = first.queue_with(authority);
    let _ = second.queue_with(authority);
}

fn main() {
    let _ = (message::Channel::Email, policy::ReviewGate::CustomerMessageApproval, Uuid::nil());
}
