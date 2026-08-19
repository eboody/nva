use domain::customer::intelligence as crm;

fn main() {
    let raw = serde_json::json!({
        "review_state": "Accepted",
        "reviewed_by": "caller-selected-reviewer"
    });
    let _accepted = serde_json::from_value::<crm::AcceptedNote>(raw).unwrap();
}
