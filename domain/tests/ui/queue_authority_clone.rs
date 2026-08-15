use domain::entities::message_record::QueueAuthorization;

fn clone_authority(authority: QueueAuthorization) {
    let _: QueueAuthorization = authority.clone();
}

fn main() {}
