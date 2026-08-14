use domain::entities::{Message, message_record};

fn queue_from_history(message: Message, evidence: message_record::ApprovalEvidence) {
    let _ = message.queue_with(evidence.queue_capability());
}

fn main() {}