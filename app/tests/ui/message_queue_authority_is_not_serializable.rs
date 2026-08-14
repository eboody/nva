use domain::entities::message_record;

fn persist_authority(authority: &message_record::QueueAuthorization) {
    let _ = serde_json::to_string(authority);
}

fn main() {}