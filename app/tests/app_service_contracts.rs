use app::booking_triage;
use domain::{care, entities, payment, policy};
use uuid::Uuid;

#[derive(Clone)]
struct FakeReservationContext {
    reservation: entities::Reservation,
}

impl booking_triage::reservation::Repository for FakeReservationContext {
    fn get(&self, id: entities::reservation::Id) -> Option<entities::Reservation> {
        (self.reservation.id == id).then(|| self.reservation.clone())
    }
}

fn reservation_with_hard_stops(hard_stops: Vec<entities::HardStop>) -> entities::Reservation {
    entities::Reservation::builder()
        .id(entities::reservation::Id(Uuid::from_u128(1)))
        .location_id(entities::LocationId(Uuid::from_u128(2)))
        .customer_id(entities::CustomerId(Uuid::from_u128(3)))
        .pet_ids(vec![entities::PetId(Uuid::from_u128(4))])
        .service(entities::ServiceKind::Boarding)
        .status(entities::reservation::Status::Requested)
        .starts_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
        .ends_at(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
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

    let packet = service.evaluate(reservation.id).unwrap();

    assert_eq!(
        packet.suggested_status(),
        entities::reservation::Status::VaccinePending
    );
    assert!(
        packet
            .deterministic_result()
            .requires(booking_triage::ApprovalGate::MedicalDocumentReview)
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

    let packet = service.evaluate(reservation.id).unwrap();

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
fn booking_triage_service_treats_paid_deposit_and_no_hard_stops_as_staff_ready() {
    let amount = domain::money::Money::new(
        domain::money::MinorUnits::try_new(2_500).unwrap(),
        domain::money::Currency::Usd,
    );
    let mut reservation = reservation_with_hard_stops(Vec::new());
    reservation.deposit = Some(payment::Deposit::paid(
        amount,
        payment::Reference::try_new("gingr-payment-123").unwrap(),
    ));
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation.clone(),
    });

    let packet = service.evaluate(reservation.id).unwrap();

    assert_eq!(
        packet.suggested_status(),
        entities::reservation::Status::Offered
    );
    assert!(
        packet
            .deterministic_result()
            .staff_may_confirm_without_human_gate()
    );
}

#[test]
fn booking_triage_service_keeps_repository_misses_as_safe_app_errors() {
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation_with_hard_stops(Vec::new()),
    });

    let missing_id = entities::reservation::Id(Uuid::from_u128(99));
    let missing = service.evaluate(missing_id);

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
    reservation
        .hard_stops
        .push(entities::HardStop::MedicalOrMedicationReviewRequired);
    let service = booking_triage::Service::new(FakeReservationContext {
        reservation: reservation.clone(),
    });

    let packet = service.evaluate(reservation.id).unwrap();

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
