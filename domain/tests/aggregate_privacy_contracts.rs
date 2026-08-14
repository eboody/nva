#[test]
fn reservation_aggregate_fields_cannot_be_bypassed_with_struct_literals() {
    let test = trybuild::TestCases::new();
    test.compile_fail("tests/ui/aggregate_literal_bypass.rs");
    test.compile_fail("tests/ui/staff_task_literal_bypass.rs");
    test.compile_fail("tests/ui/workflow_event_literal_bypass.rs");
}
