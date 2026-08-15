use domain::agent::knowledge::AuthorizedEvidence;

fn serialize(value: &AuthorizedEvidence) {
    let _ = serde_json::to_string(value).unwrap();
}

fn main() {}
