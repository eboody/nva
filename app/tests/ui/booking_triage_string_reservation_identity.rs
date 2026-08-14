use app::booking_triage as triage;

fn main() {
    let _request = triage::Request::<triage::Intake>::builder()
        .reservation(triage::Reservation::try_new("arbitrary-reservation-label").unwrap())
        .build();
}
