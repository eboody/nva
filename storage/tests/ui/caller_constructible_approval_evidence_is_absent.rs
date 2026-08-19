use storage::operations::{
    ActorKindCode, ApprovalReviewDisposition, ApprovalTargetBinding, ReviewGateCode,
};

fn main() {
    let target = ApprovalTargetBinding::new(
        "approval".to_owned(),
        "message".to_owned(),
        "target".to_owned(),
        ReviewGateCode::ManagerApproval,
    );
    let _ = ApprovalReviewDisposition::approved(
        ActorKindCode::Manager,
        "manager".to_owned(),
        "2026-08-18T00:00:00Z".to_owned(),
        None,
        target,
    );
}
