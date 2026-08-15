use domain::customer;

fn main() {
    let accepted: customer::intelligence::AcceptedNote = todo!();
    let _ = serde_json::to_value(accepted);
}
