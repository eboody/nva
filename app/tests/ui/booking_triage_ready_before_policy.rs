use app::booking_triage as triage;

fn main() {
    let request = triage::Request::<triage::Intake>::builder()
        .reservation(triage::Reservation::try_new("reservation-fixture-123").unwrap())
        .build()
    .attach_pet_profile(
        domain::pet::Name::try_new("Miso").unwrap(),
        triage::PetProfileCompleteness::Complete,
    );

    let _illegal = request.mark_ready_for_policy_decision();
}