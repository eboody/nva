use app::crm_retention::{FollowUpEligibility, IneligibilityReason};

fn main() {
    let _forged = FollowUpEligibility {
        reason: IneligibilityReason::AcceptedConsentAuthorityUnavailable,
    };
}
