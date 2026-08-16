use domain::{daycare, entities, policy, retail, source, training};
use uuid::Uuid;

fn source_ref(record_id: &str) -> source::RecordRef {
    source::RecordRef::new(
        source::System::Gingr,
        source::record::Id::try_new(record_id).expect("test source id is valid"),
    )
}

#[test]
fn training_package_opportunity_carries_source_review_blocked_actions_and_outcome_minutes() {
    let package_id = training::package::Id::try_new("pkg-123").unwrap();
    let ledger = training::package::Ledger::open(training::package::OpeningLedger {
        package_id: package_id.clone(),
        customer_id: entities::CustomerId::new(Uuid::new_v4()),
        pet_id: entities::PetId::new(Uuid::new_v4()),
        policy: training::package::Policy::MultiSessionPackage {
            sessions: training::SessionCount::try_new(1).unwrap(),
        },
        entries: vec![training::package::LedgerEntry::Consumed {
            session_id: training::SessionId::try_new("session-1").unwrap(),
        }],
    })
    .unwrap();

    let usage = training::package::UsagePolicy.decide_usage(&ledger);
    let opportunity = training::package::Opportunity::from_usage_decision(
        ledger.package_id().clone(),
        vec![source_ref("training-package-ledger-123")],
        usage,
        training::package::EstimatedLaborMinutes::try_new(7).unwrap(),
    )
    .unwrap();

    assert!(opportunity.has_source_evidence());
    assert_eq!(
        format!("{opportunity:?}"),
        "training::package::Opportunity([REDACTED])"
    );
    assert!(!format!("{opportunity:?}").contains("pkg-123"));
    assert_eq!(
        opportunity.review_gate(),
        Some(policy::ReviewGate::RefundOrDepositException)
    );
    assert!(
        opportunity
            .blocked_actions()
            .contains(&training::package::BlockedAction::MutatePackageOrPaymentBalance)
    );
    assert!(
        opportunity
            .blocked_actions()
            .contains(&training::package::BlockedAction::AssignTrainerOrSession)
    );

    let outcome = opportunity.record_outcome(
        training::package::Disposition::ReconciliationQueuedLabel,
        training::package::ReportedLaborMinutes::try_new(3).unwrap(),
    );
    assert_eq!(outcome.before_minutes().get(), 7);
    assert_eq!(outcome.reported_minutes().get(), 3);
    assert_eq!(outcome.reported_estimated_minutes_difference(), 0);
}

#[test]
fn retail_reorder_decision_exposes_review_gate_and_blocks_vendor_orders() {
    let sku = retail::product::Sku::try_new("CALM-CARE-30").unwrap();
    let position = retail::inventory::Position::record(retail::inventory::Stock {
        location_id: entities::LocationId::new(Uuid::new_v4()),
        sku,
        on_hand: retail::inventory::OnHandUnits::new(3),
        reserved: retail::inventory::ReservedUnits::new(1),
        reorder_at: retail::inventory::UnitCount::try_new(2).unwrap(),
    })
    .unwrap();

    let decision = retail::reorder::Policy::AutoCreateManagerTask.evaluate(&position);
    assert_eq!(
        decision.review_gate(),
        Some(policy::ReviewGate::ManagerApproval)
    );
    assert!(
        decision
            .blocked_actions()
            .contains(&retail::reorder::BlockedAction::PlaceVendorPurchaseOrder)
    );
    assert!(
        decision
            .blocked_actions()
            .contains(&retail::reorder::BlockedAction::MutateInventoryOrPos)
    );
}

#[test]
fn daycare_package_opportunity_keeps_source_refs_and_records_labor_outcome() {
    let private_customer = Uuid::parse_str("86753090-1111-4111-8111-111111111111").unwrap();
    let private_pet = Uuid::parse_str("86753090-2222-4222-8222-222222222222").unwrap();
    let evidence = daycare::package_opportunity::Evidence::builder()
        .customer_id(entities::CustomerId::new(private_customer))
        .pet_id(entities::PetId::new(private_pet))
        .attendance_visits(daycare::package_opportunity::AttendanceVisitCount::new(8))
        .eligibility(daycare::package_opportunity::CareEligibility::Cleared)
        .package_state(daycare::package_opportunity::PackageState::PayPerVisit)
        .payment_state(daycare::package_opportunity::PaymentState::Current)
        .source_record_refs(vec![source_ref("daycare-attendance-123")])
        .build();

    let decision = daycare::package_opportunity::Policy.classify(&evidence);
    assert!(evidence.has_source_evidence());
    assert_eq!(
        format!("{evidence:?}"),
        "daycare::package_opportunity::Evidence([REDACTED])"
    );
    assert!(!format!("{evidence:?}").contains(&private_customer.to_string()));
    assert!(!format!("{evidence:?}").contains(&private_pet.to_string()));
    assert_eq!(
        decision.review_gate(),
        Some(policy::ReviewGate::CustomerMessageApproval)
    );
    assert!(
        decision
            .blocked_actions()
            .contains(&daycare::package_opportunity::BlockedAction::EnrollPackageOrMembership)
    );

    let outcome = daycare::package_opportunity::OutcomeRecord::new(
        daycare::package_opportunity::Disposition::StaffReviewedOffer,
        daycare::package_opportunity::EstimatedLaborMinutes::try_new(6).unwrap(),
        daycare::package_opportunity::ActualLaborMinutes::try_new(2).unwrap(),
        evidence.source_record_refs().to_vec(),
    );
    assert_eq!(outcome.reported_estimated_minutes_difference(), 0);
    assert!(outcome.has_source_evidence());
}
