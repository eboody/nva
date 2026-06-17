use app::local_smoke;

#[test]
fn local_demo_smoke_proves_review_gated_inquiry_to_retention_chain() {
    let fixture = include_str!("../../fixtures/smoke/inquiry-received.json");

    let evidence = local_smoke::run_fixture(fixture).expect("local full-chain smoke fixture runs");

    assert_eq!(
        evidence.source_event_key().as_ref(),
        "local-smoke-inquiry-001"
    );
    assert_eq!(
        evidence.stage_names(),
        vec![
            "inquiry",
            "profile",
            "vaccine_docs",
            "booking_triage",
            "confirmation_draft",
            "check_in_today_view",
            "staff_note_daily_update_draft",
            "checkout_completion",
            "follow_up_retention",
        ]
    );
    assert!(evidence.boundaries().draft_only_ai());
    assert!(evidence.boundaries().blocks_live_customer_sends());
    assert!(evidence.boundaries().blocks_provider_or_pms_mutations());
    assert!(
        evidence
            .boundaries()
            .blocks_payment_refund_or_discount_actions()
    );
    assert!(
        evidence
            .confirmation_draft()
            .requires_customer_message_approval()
    );
    assert!(
        evidence
            .daily_update_preview()
            .send_stub
            .is_blocked_until_human_approval()
    );
    assert_eq!(
        evidence.booking_packet().suggested_status(),
        domain::entities::reservation::Status::Offered
    );
    assert!(
        evidence
            .today_view()
            .reservation_labels()
            .iter()
            .any(|label| label.as_ref() == "REQ-local-smoke-inquiry-001")
    );
    assert_eq!(
        *evidence.today_view().status(),
        domain::entities::reservation::Status::CheckedIn
    );
    assert_eq!(
        evidence.checkout_completion().status(),
        domain::entities::reservation::Status::CheckedOut
    );
    assert_eq!(
        evidence.checkout_completion().completion_status(),
        app::checkout_completion::CompletionStatus::StaffVerifiedCheckout
    );
    assert_eq!(
        evidence.checkout_completion().required_review_gates(),
        &[domain::policy::ReviewGate::CustomerMessageApproval]
    );
    assert!(
        evidence
            .checkout_completion()
            .blocked_actions()
            .contains(&app::checkout_completion::BlockedAction::SendCustomerMessage)
    );
    assert!(
        evidence
            .checkout_completion()
            .blocked_actions()
            .contains(&app::checkout_completion::BlockedAction::MutateProviderOrPmsRecord)
    );
    assert_eq!(
        evidence.retention_follow_up().next_action(),
        local_smoke::RetentionNextAction::DraftRebookingReminderForReview
    );
    assert_eq!(
        evidence.retention_follow_up().review_gate(),
        domain::policy::ReviewGate::CustomerMessageApproval
    );
    assert!(
        evidence
            .review_gated_evidence_refs()
            .iter()
            .any(|e| e.as_ref() == "vaccine_docs:medical_document_review_required")
    );
    assert!(
        evidence
            .review_gated_evidence_refs()
            .iter()
            .any(|e| { e.as_ref() == "checkout_completion:customer_message_approval_required" })
    );
}
