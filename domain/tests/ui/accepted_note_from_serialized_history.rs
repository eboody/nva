use domain::customer;

fn main() {
    let raw = serde_json::json!({});
    let _ = serde_json::from_value::<customer::intelligence::AcceptedNote>(raw);
}
