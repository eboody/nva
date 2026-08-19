use storage::workflow_repository::PostgresSourceQualityBacklog;

#[test]
fn postgres_source_quality_backlog_debug_redacts_database_credentials() {
    let credential = "postgres://reviewer:card20-secret@database.internal/nva";
    let adapter = PostgresSourceQualityBacklog::new(credential);

    let debug = format!("{adapter:?}");

    assert!(!debug.contains(credential));
    assert!(!debug.contains("reviewer"));
    assert!(!debug.contains("card20-secret"));
    assert!(debug.contains("<redacted>"));
}
