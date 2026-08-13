#[test]
fn booking_triage_typestate_blocks_policy_decision_before_source_evidence_order() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/booking_triage_policy_before_pet_profile.rs");
    tests.compile_fail("tests/ui/booking_triage_ready_before_policy.rs");
}
