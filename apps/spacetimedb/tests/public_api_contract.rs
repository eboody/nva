#[test]
fn reported_outcome_card_has_one_canonical_public_path() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/hygiene_outcome_card_uses_canonical_path.rs");
}
