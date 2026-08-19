use app::crm_retention::FollowUpEligibility;

fn serialize(value: &FollowUpEligibility) {
    let _ = serde_json::to_string(value);
}

fn main() {}
