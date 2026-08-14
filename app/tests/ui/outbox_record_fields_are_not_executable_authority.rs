use storage::operations::OutboxRecord;

fn rewrite_persisted_candidate(mut record: OutboxRecord) {
    record.topic = "provider.live_write".to_owned();
}

fn main() {}