use app::booking_triage as triage;

fn main() {
    let request = triage::Request::<triage::Intake>::builder()
        .reservation(domain::entities::reservation::Id(uuid::Uuid::from_u128(
            123,
        )))
        .build();

    let _illegal = request
        .attach_policy_snapshot(triage::PolicySnapshot::try_new("policy:boarding:v1").unwrap());
}
