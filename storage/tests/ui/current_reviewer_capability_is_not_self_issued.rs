use storage::operations::{ActorKindCode, CurrentApprovalReviewerCapability};

fn main() {
    let _ = CurrentApprovalReviewerCapability::try_new(
        ActorKindCode::Manager,
        "self-asserted-manager".to_owned(),
    );
}
