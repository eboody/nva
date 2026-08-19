use app::checkout_completion;

fn main() {
    let _: checkout_completion::ReviewPacket = serde_json::from_str("{}").unwrap();
}
