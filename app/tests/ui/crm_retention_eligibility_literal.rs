use app::crm_retention::{EligibilityReason, FollowUpEligibility};

fn main() {
    let _forged = FollowUpEligibility::Eligible {
        reason: EligibilityReason::SourceGroundedRetentionOpportunity,
    };
}
