use app::booking_triage as triage;

fn main() {
    let request = triage::Request::<triage::Intake>::builder()
        .reservation(triage::Reservation::try_new("reservation-fixture-123").unwrap())
        .build();

    let _illegal = request.attach_policy_snapshot(
        triage::PolicySnapshot::try_new("policy:boarding:v1").unwrap(),
    );
}