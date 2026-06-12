use app::booking_triage;
use domain::{boarding, money, payment, policy};

fn evidence(label: &str) -> booking_triage::EvidenceRef {
    booking_triage::EvidenceRef::try_new(label).unwrap()
}

#[test]
fn booking_triage_rule_contract_composes_boarding_deposit_decision_without_owning_it() {
    let amount = money::Money::new(
        money::MinorUnits::try_new(2_500).unwrap(),
        money::Currency::Usd,
    );
    let readiness = boarding::deposit::Policy::new(
        boarding::DepositRule::Required { amount },
        boarding::PaymentTiming::DueAtBooking,
    )
    .readiness_for_confirmation(None);

    let rule = match readiness {
        boarding::deposit::ConfirmationReadiness::Ready => booking_triage::RuleEvaluation::pass(
            booking_triage::RuleId::DepositAndPricingRequirements,
            vec![evidence("deposit:policy:due-at-booking")],
        ),
        boarding::deposit::ConfirmationReadiness::Blocked { review_gate, .. } => {
            booking_triage::RuleEvaluation::needs_human_approval(
                booking_triage::RuleId::DepositAndPricingRequirements,
                booking_triage::FailureCode::DepositNotSatisfied,
                booking_triage::ReadinessBucket::SpecialReview,
                triage_gate_for(review_gate),
                vec![evidence("deposit:policy:due-at-booking")],
            )
        }
    };

    let evaluation = booking_triage::DeterministicResult::evaluate(vec![rule]);

    assert_eq!(
        evaluation.recommended_status(),
        booking_triage::ReadinessBucket::SpecialReview
    );
    assert!(evaluation.requires(booking_triage::ApprovalGate::PaymentManagerApproval));
    assert!(
        evaluation
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::MovePayment)
    );
}

#[test]
fn booking_triage_rule_contract_treats_paid_boarding_deposit_as_ready_service_evidence() {
    let amount = money::Money::new(
        money::MinorUnits::try_new(2_500).unwrap(),
        money::Currency::Usd,
    );
    let paid = payment::Deposit::paid(
        amount.clone(),
        payment::PaymentReference::try_new("gingr-payment-123").unwrap(),
    );
    let readiness = boarding::deposit::Policy::new(
        boarding::DepositRule::Required { amount },
        boarding::PaymentTiming::DueAtBooking,
    )
    .readiness_for_confirmation(Some(&paid));

    assert_eq!(readiness, boarding::deposit::ConfirmationReadiness::Ready);

    let evaluation =
        booking_triage::DeterministicResult::evaluate(vec![booking_triage::RuleEvaluation::pass(
            booking_triage::RuleId::DepositAndPricingRequirements,
            vec![evidence("payment:gingr-payment-123")],
        )]);

    assert_eq!(
        evaluation.recommended_status(),
        booking_triage::ReadinessBucket::ReadyForStaffApproval
    );
    assert!(!evaluation.requires(booking_triage::ApprovalGate::PaymentManagerApproval));
}

fn triage_gate_for(gate: policy::ReviewGate) -> booking_triage::ApprovalGate {
    match gate {
        policy::ReviewGate::ManagerApproval => booking_triage::ApprovalGate::ManagerApproval,
        policy::ReviewGate::MedicalDocumentReview => {
            booking_triage::ApprovalGate::MedicalDocumentReview
        }
        policy::ReviewGate::BehaviorReview => booking_triage::ApprovalGate::BehaviorReview,
        policy::ReviewGate::RefundOrDepositException => {
            booking_triage::ApprovalGate::PaymentManagerApproval
        }
        policy::ReviewGate::CustomerMessageApproval => {
            booking_triage::ApprovalGate::CustomerMessageApproval
        }
    }
}
