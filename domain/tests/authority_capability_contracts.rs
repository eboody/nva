#[test]
fn executable_authority_cannot_be_manufactured_or_replayed_by_callers() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/queue_authority_from_history.rs");
    tests.compile_fail("tests/ui/lead_queue_authority_literal.rs");
    tests.compile_fail("tests/ui/authorized_knowledge_evidence_has_no_public_issuer.rs");
    tests.compile_fail("tests/ui/authorized_knowledge_evidence_clone.rs");
    tests.compile_fail("tests/ui/authorized_knowledge_evidence_serialize.rs");
    tests.compile_fail("tests/ui/queue_authority_literal.rs");
    tests.compile_fail("tests/ui/queue_authority_clone.rs");
    tests.compile_fail("tests/ui/queue_authority_serialize.rs");
    tests.compile_fail("tests/ui/queue_authority_reuse.rs");
    tests.compile_fail("tests/ui/incident_closure_from_history.rs");
    tests.compile_fail("tests/ui/marketing_permission_from_consent_evidence.rs");
    tests.compile_fail("tests/ui/accepted_note_from_serialized_history.rs");
    tests.compile_fail("tests/ui/compatibility_alias_cannot_mint_accepted_note.rs");
    tests.compile_fail("tests/ui/accepted_note_clone.rs");
    tests.compile_fail("tests/ui/accepted_note_serialize.rs");
}
