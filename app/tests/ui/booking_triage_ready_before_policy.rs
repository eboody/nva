use app::booking_triage as triage;

fn main() {
    let request = triage::Request::<triage::Intake>::builder()
        .reservation(domain::entities::reservation::Id(uuid::Uuid::from_u128(
            123,
        )))
        .build()
        .attach_pet_profile(
            domain::pet::Name::try_new("Miso").unwrap(),
            triage::PetProfileCompleteness::Complete,
        );

    let _illegal = request.mark_ready_for_policy_decision();
}
