use domain::{entities, identity, policy};

#[test]
fn play_eligibility_rejects_contradictory_eligible_review_state() {
    let contradictory = serde_json::json!({
        "eligibility": { "Eligible": "NoConservativeHardStop" },
        "required_review": "BehaviorReview"
    });

    assert!(
        serde_json::from_value::<policy::play::Decision>(contradictory).is_err(),
        "an eligible decision cannot simultaneously require review"
    );
}

#[test]
fn ambiguous_identity_rehydration_requires_two_unique_candidates() {
    let customer = entities::CustomerId::new(uuid::Uuid::from_u128(1));
    let one = serde_json::json!({ "Ambiguous": { "candidates": [customer] } });
    let duplicates = serde_json::json!({
        "Ambiguous": { "candidates": [customer, customer] }
    });

    assert!(serde_json::from_value::<identity::Match>(one).is_err());
    assert!(serde_json::from_value::<identity::Match>(duplicates).is_err());
}
