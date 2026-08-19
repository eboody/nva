use std::fs;
use std::path::{Path, PathBuf};

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn physical_lines(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count()
}

#[test]
fn oversized_domain_workflows_are_thin_indexes_over_concept_owners() {
    let source_root = source_root();
    let families = [
        (
            "training/mod.rs",
            "training",
            &[
                "program.rs",
                "enrollment.rs",
                "curriculum.rs",
                "trainer.rs",
                "progress.rs",
                "outcome.rs",
                "package.rs",
                "follow_up.rs",
                "error.rs",
            ][..],
        ),
        (
            "operations.rs",
            "operations",
            &[
                "labor.rs",
                "capacity.rs",
                "time_bucket.rs",
                "operating_day.rs",
                "operating_window.rs",
                "reporting_period.rs",
                "operational.rs",
                "pet_resort.rs",
                "lodging_offer.rs",
                "service_core.rs",
            ][..],
        ),
        (
            "source.rs",
            "source",
            &["record.rs", "reservation.rs", "error.rs"][..],
        ),
        (
            "workflow.rs",
            "workflow",
            &[
                "external.rs",
                "task.rs",
                "message.rs",
                "status_update.rs",
                "event.rs",
                "outcome.rs",
            ][..],
        ),
    ];

    for (root_name, owner_dir, owners) in families {
        let root = source_root.join(root_name);
        assert!(
            physical_lines(&root) < 500,
            "{} must be a thin index below 500 physical lines",
            root.display()
        );

        for owner in owners {
            let path = source_root.join(owner_dir).join(owner);
            assert!(path.is_file(), "missing concept owner {}", path.display());
            assert!(
                physical_lines(&path) < 1_500,
                "concept owner {} must stay below 1,500 physical lines",
                path.display()
            );
        }
    }

    assert!(
        !source_root.join("training/availability.rs").exists(),
        "retired consumerless training availability model must not return"
    );
}

#[test]
fn domain_workflow_roots_use_named_reexports_without_inline_owner_modules() {
    for root_name in [
        "training/mod.rs",
        "operations.rs",
        "source.rs",
        "workflow.rs",
    ] {
        let root = fs::read_to_string(source_root().join(root_name)).unwrap();
        assert!(
            !root.contains("pub use ") || !root.contains("::*"),
            "{root_name} must not flatten ownership through broad re-exports"
        );
        for owner in [
            "program",
            "enrollment",
            "curriculum",
            "trainer",
            "progress",
            "outcome",
            "package",
            "follow_up",
            "labor",
            "capacity",
            "time_bucket",
            "operating_day",
            "operating_window",
            "reporting_period",
            "operational",
            "pet_resort",
            "lodging_offer",
            "service_core",
            "record",
            "reservation",
            "gingr",
            "external",
            "task",
            "message",
            "status_update",
        ] {
            assert!(
                !root.contains(&format!("pub mod {owner} {{")),
                "{root_name} still implements {owner} inline"
            );
        }
    }
}
