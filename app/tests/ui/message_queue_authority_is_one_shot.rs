use domain::entities::{Message, message_record};

fn queue_twice(
    first: Message,
    second: Message,
    authority: message_record::QueueAuthorization,
) {
    let _ = first.queue_with(authority);
    let _ = second.queue_with(authority);
}

fn main() {}