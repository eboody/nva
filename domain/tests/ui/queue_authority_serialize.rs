use domain::entities::message_record::QueueAuthorization;

fn serialize(authority: &QueueAuthorization) {
    let _ = serde_json::to_string(authority);
}

fn main() {}
