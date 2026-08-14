use storage::operations::{InternalHandoffTopic, PendingOutboxRecord};

fn rewrite_admitted_candidate(mut record: PendingOutboxRecord) {
    record.topic = InternalHandoffTopic::SiteFinanceReviewedHandoff;
}

fn main() {}