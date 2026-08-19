use std::{fs, path::PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app crate has a workspace parent")
        .to_path_buf()
}

fn physical_lines(path: &str) -> usize {
    let source = fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    source.lines().count()
}

#[test]
fn app_workflow_roots_route_to_concept_owned_modules() {
    let expected_modules = [
        "app/src/data_quality_hygiene/workflow.rs",
        "app/src/data_quality_hygiene/outcome.rs",
        "app/src/data_quality_hygiene/outcome_capture.rs",
        "app/src/booking_triage/workflow.rs",
        "app/src/booking_triage/service.rs",
        "app/src/manager_daily_brief/workflow.rs",
        "app/src/manager_daily_brief/outcome.rs",
    ];

    for path in expected_modules {
        assert!(
            workspace_root().join(path).is_file(),
            "missing concept-owned app module {path}"
        );
        assert!(
            physical_lines(path) < 1_500,
            "concept-owned app module {path} must remain below 1,500 physical lines"
        );
    }

    for retired_speculative_tool_module in [
        "availability.rs",
        "documents.rs",
        "draft_update.rs",
        "hermes.rs",
        "media.rs",
        "messaging.rs",
        "payment.rs",
        "portal.rs",
    ] {
        assert!(
            !workspace_root()
                .join("app/src/tools")
                .join(retired_speculative_tool_module)
                .exists(),
            "retired speculative tool module must not return: {retired_speculative_tool_module}"
        );
    }

    for root in [
        "app/src/data_quality_hygiene.rs",
        "app/src/booking_triage.rs",
        "app/src/manager_daily_brief.rs",
        "app/src/tools.rs",
    ] {
        assert!(
            physical_lines(root) < 500,
            "app workflow root {root} must remain routing/composition glue below 500 physical lines"
        );
    }
}
