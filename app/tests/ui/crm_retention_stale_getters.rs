use app::crm_retention::{OutcomeRecord, Packet, StaffReviewPacket};

fn stale_getters(review: &StaffReviewPacket, outcome: &OutcomeRecord, packet: &Packet) {
    let _ = review.marketable_opportunities();
    let _ = outcome.records_staff_evidence_only();
    let _ = outcome.blocked_actions();
    let _ = outcome.matches_reported_packet_evidence(packet);
}

fn main() {}
