use domain::agent::knowledge::AuthorizedEvidence;

fn duplicate(value: &AuthorizedEvidence) -> AuthorizedEvidence {
    (*value).clone()
}

fn main() {}
