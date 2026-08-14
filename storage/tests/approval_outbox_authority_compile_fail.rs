#[test]
fn approved_internal_handoff_authority_is_opaque_one_shot_and_not_serde() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/current_reviewer_capability_is_not_self_issued.rs");
    tests.compile_fail("tests/ui/approved_internal_handoff_authority_is_opaque.rs");
    tests.compile_fail("tests/ui/approved_internal_handoff_authority_is_not_clone.rs");
    tests.compile_fail("tests/ui/approved_internal_handoff_authority_is_not_serializable.rs");
    tests.compile_fail("tests/ui/approval_evidence_is_not_pending_outbox_authority.rs");
}
