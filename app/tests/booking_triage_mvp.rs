use app::booking_triage;
use domain::entities;

fn evidence(label: &str) -> booking_triage::EvidenceRef {
    booking_triage::EvidenceRef::try_new(label).unwrap()
}

fn finding(
    rule_id: booking_triage::rule::Id,
    failure_code: booking_triage::FailureCode,
    readiness_bucket: booking_triage::ReadinessBucket,
    human_approval_required: booking_triage::ApprovalGate,
    evidence_refs: Vec<booking_triage::EvidenceRef>,
) -> booking_triage::rule::ReviewFinding {
    booking_triage::rule::ReviewFinding::builder()
        .rule_id(rule_id)
        .failure_code(failure_code)
        .readiness_bucket(readiness_bucket)
        .human_approval_required(human_approval_required)
        .evidence_refs(evidence_refs)
        .build()
}

#[test]
fn deterministic_gates_block_confirmation_when_vaccine_review_is_pending() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::AccommodationAvailability,
            vec![evidence("availability:snapshot:cap-123")],
        ),
        booking_triage::rule::Evaluation::unknown(finding(
            booking_triage::rule::Id::VaccineRequirements,
            booking_triage::FailureCode::MissingOrUnverifiedVaccine,
            booking_triage::ReadinessBucket::VaccinePending,
            booking_triage::ApprovalGate::MedicalDocumentReview,
            vec![evidence("document:rabies-upload-77")],
        )),
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
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::DateRangeAndServiceSupported,
            vec![evidence("policy:service-catalog:v1")],
        ),
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::AccommodationAvailability,
            vec![evidence("availability:snapshot:cap-123")],
        ),
        booking_triage::rule::Evaluation::pass(
            booking_triage::rule::Id::DepositAndPricingRequirements,
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
        entities::reservation::Status::Offered
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
        booking_triage::rule::Evaluation::needs_human_approval(finding(
            booking_triage::rule::Id::BehaviorRestrictions,
            booking_triage::FailureCode::BehaviorExceptionRequiresReview,
            booking_triage::ReadinessBucket::SpecialReview,
            booking_triage::ApprovalGate::BehaviorReview,
            vec![evidence("incident:restriction-15")],
        )),
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
        booking_triage::rule::Evaluation::unknown(finding(
            booking_triage::rule::Id::VaccineRequirements,
            booking_triage::FailureCode::MissingOrUnverifiedVaccine,
            booking_triage::ReadinessBucket::VaccinePending,
            booking_triage::ApprovalGate::MedicalDocumentReview,
            vec![evidence("document:rabies-upload-77")],
        )),
        booking_triage::rule::Evaluation::hard_block(finding(
            booking_triage::rule::Id::HolidayBlackoutMinimumStay,
            booking_triage::FailureCode::PolicyHardStop,
            booking_triage::ReadinessBucket::Rejected,
            booking_triage::ApprovalGate::RejectionApproval,
            vec![evidence("policy:blackout:min-stay")],
        )),
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
        entities::reservation::Status::SpecialReview
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
        booking_triage::rule::Evaluation::needs_human_approval(finding(
            booking_triage::rule::Id::MedicationSpecialCareLimits,
            booking_triage::FailureCode::SpecialCareRequiresReview,
            booking_triage::ReadinessBucket::SpecialReview,
            booking_triage::ApprovalGate::CareTeamApproval,
            vec![evidence("care-plan:medication-review")],
        )),
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

#[test]
fn missing_information_reason_stays_source_backed_and_blocks_unsafe_actions() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::rule::Evaluation::unknown(
            finding(
                booking_triage::rule::Id::DateRangeAndServiceSupported,
                booking_triage::FailureCode::MissingRequiredInput,
                booking_triage::ReadinessBucket::MissingInfo,
                booking_triage::ApprovalGate::StaffApproval,
                vec![evidence("website-form:req-44:missing-end-date")],
            )
            .with_missing_info_reason(booking_triage::MissingInfoReason::RequestedDateWindow),
        ),
    ]);

    assert_eq!(
        evaluation.recommended_status(),
        booking_triage::ReadinessBucket::MissingInfo
    );
    assert_eq!(
        evaluation.missing_info_reasons(),
        &[booking_triage::MissingInfoReason::RequestedDateWindow]
    );
    assert!(
        evaluation
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::MutateProviderRecord)
    );
    assert!(
        evaluation
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::SendCustomerMessage)
    );
    assert!(
        evaluation
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::MovePayment)
    );
}

#[test]
fn blocker_evidence_classifies_care_vaccine_payment_and_policy_review_pressure() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::rule::Evaluation::unknown(
            finding(
                booking_triage::rule::Id::VaccineRequirements,
                booking_triage::FailureCode::MissingOrUnverifiedVaccine,
                booking_triage::ReadinessBucket::VaccinePending,
                booking_triage::ApprovalGate::MedicalDocumentReview,
                vec![evidence("vaccine:rabies:expired")],
            )
            .with_blocker_kind(booking_triage::BlockerKind::Vaccine),
        ),
        booking_triage::rule::Evaluation::needs_human_approval(
            finding(
                booking_triage::rule::Id::MedicationSpecialCareLimits,
                booking_triage::FailureCode::SpecialCareRequiresReview,
                booking_triage::ReadinessBucket::SpecialReview,
                booking_triage::ApprovalGate::CareTeamApproval,
                vec![evidence("care:medication-review")],
            )
            .with_blocker_kind(booking_triage::BlockerKind::Care),
        ),
        booking_triage::rule::Evaluation::needs_human_approval(
            finding(
                booking_triage::rule::Id::DepositAndPricingRequirements,
                booking_triage::FailureCode::DepositNotSatisfied,
                booking_triage::ReadinessBucket::SpecialReview,
                booking_triage::ApprovalGate::PaymentManagerApproval,
                vec![evidence("deposit:unpaid")],
            )
            .with_blocker_kind(booking_triage::BlockerKind::Payment),
        ),
        booking_triage::rule::Evaluation::hard_block(
            finding(
                booking_triage::rule::Id::HolidayBlackoutMinimumStay,
                booking_triage::FailureCode::PolicyHardStop,
                booking_triage::ReadinessBucket::Rejected,
                booking_triage::ApprovalGate::ManagerApproval,
                vec![evidence("policy:holiday-blackout")],
            )
            .with_blocker_kind(booking_triage::BlockerKind::Policy),
        ),
    ]);

    let kinds: Vec<_> = evaluation
        .blocker_evidence()
        .iter()
        .map(|evidence| evidence.kind)
        .collect();

    assert_eq!(
        kinds,
        vec![
            booking_triage::BlockerKind::Vaccine,
            booking_triage::BlockerKind::Care,
            booking_triage::BlockerKind::Payment,
            booking_triage::BlockerKind::Policy,
        ]
    );
    assert!(evaluation.blocker_evidence().iter().any(|evidence| {
        evidence
            .evidence_refs
            .contains(&evidence_ref("deposit:unpaid"))
    }));
}

#[test]
fn missing_info_draft_is_reviewable_copy_not_confirmation_or_customer_send_authority() {
    let evaluation = booking_triage::DeterministicResult::evaluate(vec![
        booking_triage::rule::Evaluation::unknown(
            finding(
                booking_triage::rule::Id::DateRangeAndServiceSupported,
                booking_triage::FailureCode::MissingRequiredInput,
                booking_triage::ReadinessBucket::MissingInfo,
                booking_triage::ApprovalGate::StaffApproval,
                vec![evidence("website-form:req-45:missing-pet-name")],
            )
            .with_missing_info_reason(booking_triage::MissingInfoReason::PetProfile),
        ),
    ]);
    let packet = booking_triage::StaffEvaluationPacket::new(
        booking_triage::Reservation::try_new("REQ-45").unwrap(),
        evaluation,
    );

    assert_eq!(
        packet
            .clone()
            .try_with_confirmation_draft(booking_triage::ConfirmationDraft::new(
                booking_triage::CustomerMessageDraft::try_new("Please confirm your booking.")
                    .unwrap(),
            )),
        Err(booking_triage::ConfirmationDraftError::DeterministicGateNotReadyForDraft)
    );

    let packet = packet
        .try_with_missing_info_draft(booking_triage::MissingInfoDraft::new(
            booking_triage::CustomerMessageDraft::try_new(
                "Draft only: please provide your pet profile before staff can review dates.",
            )
            .unwrap(),
        ))
        .unwrap();

    assert_eq!(
        packet.missing_info_draft().approval_gate(),
        booking_triage::ApprovalGate::CustomerMessageApproval
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&booking_triage::AuditEventDraft::MissingInfoDraftGenerated)
    );
    assert!(
        packet
            .deterministic_result()
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::SendCustomerMessage)
    );
}

fn evidence_ref(label: &str) -> booking_triage::EvidenceRef {
    booking_triage::EvidenceRef::try_new(label).unwrap()
}
