use app::booking_triage;
use domain::{care, entities, payment, policy};
use uuid::Uuid;

#[derive(Clone)]
struct FakeReservationContext {
    reservation: entities::Reservation,
}

impl booking_triage::reservation::Repository for FakeReservationContext {
    fn get(&self, id: entities::reservation::Id) -> Option<entities::Reservation> {
        (self.reservation.id() == id).then(|| self.reservation.clone())
    }
}

fn reservation_with_hard_stops(hard_stops: Vec<entities::HardStop>) -> entities::Reservation {
    entities::Reservation::builder()
        .id(entities::reservation::Id::new(uuid::Uuid::from_u128(1)))
        .location_id(entities::LocationId::new(Uuid::from_u128(2)))
        .customer_id(entities::CustomerId::new(Uuid::from_u128(3)))
        .pet_ids(vec![entities::PetId::new(Uuid::from_u128(4))])
        .service(entities::ServiceKind::Boarding)
        .status(entities::reservation::Status::Requested)
        .starts_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .ends_at(chrono::DateTime::<chrono::Utc>::from_timestamp(60, 0).unwrap())
        .deposit(payment::Deposit::paid(
            domain::money::Money::new(
                domain::money::MinorUnits::try_new(2_500).unwrap(),
                domain::money::Currency::Usd,
            ),
            payment::Reference::try_new("test-paid-deposit").unwrap(),
        ))
        .source(entities::reservation::Source::WebsiteForm)
        .hard_stops(hard_stops)
        .build()
        .unwrap()
}

fn ready_request(
    reservation_id: entities::reservation::Id,
) -> booking_triage::Request<booking_triage::ReadyForPolicyDecision> {
    booking_triage::Request::<booking_triage::Intake>::builder()
        .reservation(reservation_id)
        .build()
        .attach_pet_profile(
            domain::pet::Name::try_new("Miso").unwrap(),
            booking_triage::PetProfileCompleteness::Complete,
        )
        .attach_policy_snapshot(
            booking_triage::PolicySnapshot::try_new("policy:boarding:v1").unwrap(),
        )
        .mark_ready_for_policy_decision()
}

#[test]
fn booking_triage_service_uses_app_repository_port_and_blocks_missing_vaccine_booking() {
    let reservation =
        reservation_with_hard_stops(vec![entities::HardStop::MissingRequiredVaccine(
            policy::VaccineName::try_new("Rabies").unwrap(),
        )]);
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation.clone(),
    });

    let packet = service.evaluate(ready_request(reservation.id())).unwrap();

    assert_eq!(
        packet.suggested_status(),
        entities::reservation::Status::SpecialReview
    );
    assert!(
        packet
            .deterministic_result()
            .requires(booking_triage::ApprovalGate::MedicalDocumentReview)
    );
    assert!(
        packet
            .deterministic_result()
            .requires(booking_triage::ApprovalGate::PaymentManagerApproval)
    );
    assert!(
        packet
            .deterministic_result()
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::ConfirmBooking)
    );
    assert!(
        packet
            .audit_event_drafts()
            .contains(&booking_triage::AuditEventDraft::PolicyDecisionRecorded)
    );
}

#[test]
fn booking_triage_service_routes_special_care_to_review_packet_without_confirmation_draft() {
    let reservation =
        reservation_with_hard_stops(vec![entities::HardStop::MedicalOrMedicationReviewRequired]);
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation.clone(),
    });

    let packet = service.evaluate(ready_request(reservation.id())).unwrap();

    assert_eq!(
        packet.deterministic_result().staff_decision_boundary(),
        booking_triage::StaffDecisionBoundary::ReviewPacketOnly
    );
    assert!(
        packet
            .deterministic_result()
            .requires(booking_triage::ApprovalGate::CareTeamApproval)
    );
    assert!(
        packet
            .deterministic_result()
            .blocked_actions()
            .contains(&booking_triage::BlockedAction::AcceptSpecialCare)
    );
}

#[test]
fn booking_triage_service_keeps_paid_deposit_and_no_hard_stops_under_staff_review() {
    let amount = domain::money::Money::new(
        domain::money::MinorUnits::try_new(2_500).unwrap(),
        domain::money::Currency::Usd,
    );
    let mut reservation = reservation_with_hard_stops(Vec::new());
    reservation = entities::Reservation::builder()
        .id(reservation.id())
        .location_id(reservation.location_id())
        .customer_id(reservation.customer_id())
        .pet_ids(reservation.pet_ids().to_vec())
        .service(reservation.service().clone())
        .status(reservation.status().clone())
        .starts_at(reservation.starts_at())
        .ends_at(reservation.ends_at())
        .deposit(payment::Deposit::paid(
            amount,
            payment::Reference::try_new("gingr-payment-123").unwrap(),
        ))
        .source(reservation.source().clone())
        .build()
        .unwrap();
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation.clone(),
    });

    let packet = service.evaluate(ready_request(reservation.id())).unwrap();

    assert_eq!(
        packet.suggested_status(),
        entities::reservation::Status::SpecialReview
    );
    assert!(
        !packet
            .deterministic_result()
            .staff_may_confirm_without_human_gate()
    );
}

#[test]
fn booking_triage_service_keeps_repository_misses_as_safe_app_errors() {
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation_with_hard_stops(Vec::new()),
    });

    let missing_id = entities::reservation::Id::new(Uuid::from_u128(99));
    let missing = service.evaluate(ready_request(missing_id));

    assert_eq!(
        missing,
        Err(booking_triage::Error::ReservationNotFound {
            reservation_id: missing_id,
        })
    );
    assert!(
        missing
            .unwrap_err()
            .to_string()
            .contains(&missing_id.to_string())
    );
}

#[test]
fn booking_triage_service_maps_care_profile_review_pressure_to_care_team_gate() {
    let mut reservation = reservation_with_hard_stops(Vec::new());
    reservation = entities::Reservation::builder()
        .id(reservation.id())
        .location_id(reservation.location_id())
        .customer_id(reservation.customer_id())
        .pet_ids(reservation.pet_ids().to_vec())
        .service(reservation.service().clone())
        .status(reservation.status().clone())
        .starts_at(reservation.starts_at())
        .ends_at(reservation.ends_at())
        .deposit(reservation.deposit().cloned().unwrap())
        .source(reservation.source().clone())
        .hard_stop(entities::HardStop::MedicalOrMedicationReviewRequired)
        .build()
        .unwrap();
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation.clone(),
    });

    let packet = service.evaluate(ready_request(reservation.id())).unwrap();

    assert!(
        packet
            .deterministic_result()
            .rule_evaluations()
            .iter()
            .any(|rule| rule.failure_code
                == Some(booking_triage::FailureCode::SpecialCareRequiresReview))
    );
    assert!(
        care::MedicationReviewRequirement::RequiresReview {
            reason: care::ReviewReason::try_new("new medication").unwrap(),
        }
        .requires_review()
    );
}
