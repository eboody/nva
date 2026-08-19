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
fn entity_root_is_a_thin_index_over_concept_owned_modules() {
    let source_root = source_root();
    let root = source_root.join("entities.rs");
    let owner_root = source_root.join("entities");
    let expected_owners = [
        "identifiers.rs",
        "location.rs",
        "customer.rs",
        "pet.rs",
        "reservation.rs",
        "document.rs",
        "care_note.rs",
        "approval.rs",
        "incident.rs",
        "message.rs",
        "actor.rs",
    ];

    assert!(
        physical_lines(&root) < 500,
        "domain/src/entities.rs must be a thin index below 500 physical lines"
    );
    for owner in expected_owners {
        let path = owner_root.join(owner);
        assert!(
            path.is_file(),
            "missing entity concept owner {}",
            path.display()
        );
        assert!(
            physical_lines(&path) < 1_500,
            "entity concept owner {} must stay below 1,500 physical lines",
            path.display()
        );
    }
}

#[test]
fn entity_root_uses_only_named_canonical_reexports() {
    let root = fs::read_to_string(source_root().join("entities.rs")).unwrap();

    assert!(
        !root.contains("pub use ") || !root.contains("::*"),
        "entity ownership must not be flattened through broad re-exports"
    );
    assert!(
        !root.to_ascii_lowercase().contains("compatibility"),
        "Task 11 requires no temporary compatibility entity facade"
    );
}
