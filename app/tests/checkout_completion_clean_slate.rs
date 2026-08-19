use std::fs;
use std::path::Path;

#[test]
fn checkout_completion_has_one_current_review_model() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/checkout_completion.rs"),
    )
    .expect("checkout completion source must be readable");

    for stale in [
        "BelongingsStatus",
        "DepartureNotesReview",
        "CompletionStatus",
        "StaffHandoff",
        "ReportedStaffCheckout",
        "suggested_reservation_status",
    ] {
        assert!(
            !source.contains(stale),
            "checkout completion must remove stale compatibility model {stale}"
        );
    }

    for current in ["DepartureObservation", "ReviewReason", "ReviewPacket"] {
        assert!(
            source.contains(current),
            "checkout completion must expose canonical current model {current}"
        );
    }
}
