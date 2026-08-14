#[test]
fn executable_authority_and_outbox_records_enforce_compile_time_protocols() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/historical_message_evidence_is_not_queue_authority.rs");
    tests.compile_fail("tests/ui/message_queue_authority_is_one_shot.rs");
    tests.compile_fail("tests/ui/message_queue_authority_is_not_serializable.rs");
    tests.compile_fail("tests/ui/outbox_record_fields_are_not_executable_authority.rs");
}
