use storage::operations::{ApprovalReviewDisposition, PendingOutboxRecord};

fn admit_from_evidence(evidence: ApprovalReviewDisposition) {
    let _ = PendingOutboxRecord::admit(evidence);
}

fn main() {}