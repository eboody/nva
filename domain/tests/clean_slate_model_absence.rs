use std::fs;
use std::path::Path;

#[test]
fn clean_slate_domain_models_expose_only_current_vocabulary() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let forbidden = [
        ("src/grooming/mod.rs", "RebookingCandidate"),
        ("src/reservation/mod.rs", "CheckoutCompletionDisposition"),
        ("src/document.rs", "MigrationImport"),
        ("src/money/mod.rs", "AccountingSystem"),
        ("src/operations/capacity.rs", "pub fn capacity("),
    ];

    for (path, symbol) in forbidden {
        let source =
            fs::read_to_string(manifest.join(path)).expect("domain source must be readable");
        assert!(
            !source.contains(symbol),
            "clean-slate domain source {path} must not retain dormant API {symbol}"
        );
    }
}
