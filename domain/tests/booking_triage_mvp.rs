use domain::{booking_triage, entities};

fn evidence(label: &str) -> booking_triage::EvidenceRef {
    booking_triage::EvidenceRef::try_new(label).unwrap()
}

#[test]
fn deterministic_gates_block_confirmation_when_vaccine_review_is_pending() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::RuleEvaluation::pass(
            booking_triage::RuleId::AccommodationAvailability,
            vec![evidence("availability:snapshot:cap-123")],
        ),
        booking_triage::RuleEvaluation::unknown(
            booking_triage::RuleId::VaccineRequirements,
            booking_triage::FailureCode::MissingOrUnverifiedVaccine,
            booking_triage::ReadinessBucket::VaccinePending,
            booking_triage::ApprovalGate::MedicalDocumentReview,
            vec![evidence("document:rabies-upload-77")],
        ),
    ]);

    assert_eq!(
        evaluation.recommended_status(),
        booking_triage::ReadinessBucket::VaccinePending
    );
    assert!(evaluation.requires(booking_triage::ApprovalGate::MedicalDocumentReview));
    assert!(!evaluation.staff_may_confirm_without_human_gate());
    assert!(
        evaluation
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::ConfirmBooking)
    );
    assert!(
        evaluation
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::RejectRequest)
    );
}

#[test]
fn ready_request_produces_staff_bounded_ai_recommendation_confirmation_draft_and_audit_events() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::RuleEvaluation::pass(
            booking_triage::RuleId::DateRangeAndServiceSupported,
            vec![evidence("policy:service-catalog:v1")],
        ),
        booking_triage::RuleEvaluation::pass(
            booking_triage::RuleId::AccommodationAvailability,
            vec![evidence("availability:snapshot:cap-123")],
        ),
        booking_triage::RuleEvaluation::pass(
            booking_triage::RuleId::DepositAndPricingRequirements,
            vec![evidence("deposit:paid:dep-9")],
        ),
    ]);

    let packet = booking_triage::StaffEvaluationPacket::new(
        booking_triage::Reservation::try_new("REQ-123").unwrap(),
        evaluation,
    )
    .with_ai_recommendation(booking_triage::AiRecommendation::recommend_staff_confirmation(
        booking_triage::RecommendationText::try_new(
            "Hard rules pass; staff can review and create an approved offer/confirmation draft.",
        )
        .unwrap(),
    ))
    .with_confirmation_draft(booking_triage::ConfirmationDraft::new(
        booking_triage::CustomerMessageDraft::try_new(
            "Draft only: we can prepare your booking confirmation after staff approval.",
        )
        .unwrap(),
    ));

    assert_eq!(
        packet.deterministic_result().recommended_status(),
        booking_triage::ReadinessBucket::ReadyForStaffApproval
    );
    assert_eq!(
        packet.ai_recommendation().recommended_action(),
        booking_triage::AgentRecommendedAction::DraftConfirmationForStaffApproval
    );
    assert_eq!(
        packet.confirmation_draft().approval_gate(),
        booking_triage::ApprovalGate::CustomerMessageApproval
    );
    assert_eq!(
        packet.suggested_status(),
        entities::ReservationStatus::Offered
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&booking_triage::AuditEventDraft::ReservationStatusSuggested)
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&booking_triage::AuditEventDraft::ConfirmationDraftGenerated)
    );
}

#[test]
fn behavior_exception_routes_to_special_review_and_keeps_decline_human_gated() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::RuleEvaluation::needs_human_approval(
            booking_triage::RuleId::BehaviorRestrictions,
            booking_triage::FailureCode::BehaviorExceptionRequiresReview,
            booking_triage::ReadinessBucket::SpecialReview,
            booking_triage::ApprovalGate::BehaviorReview,
            vec![evidence("incident:restriction-15")],
        ),
    ]);

    assert_eq!(
        evaluation.recommended_status(),
        booking_triage::ReadinessBucket::SpecialReview
    );
    assert!(evaluation.requires(booking_triage::ApprovalGate::BehaviorReview));
    assert!(
        evaluation
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::RejectRequest)
    );
    assert_eq!(
        evaluation.staff_decision_boundary(),
        booking_triage::StaffDecisionBoundary::ReviewPacketOnly
    );
}

#[test]
fn hard_rejections_dominate_review_buckets_but_only_suggest_special_review() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::RuleEvaluation::unknown(
            booking_triage::RuleId::VaccineRequirements,
            booking_triage::FailureCode::MissingOrUnverifiedVaccine,
            booking_triage::ReadinessBucket::VaccinePending,
            booking_triage::ApprovalGate::MedicalDocumentReview,
            vec![evidence("document:rabies-upload-77")],
        ),
        booking_triage::RuleEvaluation::hard_block(
            booking_triage::RuleId::HolidayBlackoutMinimumStay,
            booking_triage::FailureCode::PolicyHardStop,
            booking_triage::ReadinessBucket::Rejected,
            booking_triage::ApprovalGate::RejectionApproval,
            vec![evidence("policy:blackout:min-stay")],
        ),
    ]);

    let packet = booking_triage::StaffEvaluationPacket::new(
        booking_triage::Reservation::try_new("REQ-456").unwrap(),
        evaluation,
    );

    assert_eq!(
        packet.deterministic_result().recommended_status(),
        booking_triage::ReadinessBucket::Rejected
    );
    assert_eq!(
        packet.suggested_status(),
        entities::ReservationStatus::SpecialReview
    );
    assert!(
        packet
            .deterministic_result()
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::RejectRequest)
    );
}

#[test]
fn confirmation_draft_cannot_attach_until_deterministic_result_is_ready() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::RuleEvaluation::needs_human_approval(
            booking_triage::RuleId::MedicationSpecialCareLimits,
            booking_triage::FailureCode::SpecialCareRequiresReview,
            booking_triage::ReadinessBucket::SpecialReview,
            booking_triage::ApprovalGate::CareTeamApproval,
            vec![evidence("care-plan:medication-review")],
        ),
    ]);

    let packet = booking_triage::StaffEvaluationPacket::new(
        booking_triage::Reservation::try_new("REQ-789").unwrap(),
        evaluation,
    );
    let draft = booking_triage::ConfirmationDraft::new(
        booking_triage::CustomerMessageDraft::try_new("Draft only after all hard gates pass.")
            .unwrap(),
    );

    assert_eq!(
        packet.try_with_confirmation_draft(draft),
        Err(booking_triage::ConfirmationDraftError::DeterministicGateNotReadyForDraft)
    );
}
