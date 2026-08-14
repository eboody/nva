use serde_json::json;
use storage::persistence::{
    CurrencyCode, Id, IdempotencyKey, MoneyColumns, Period, SourceRefColumns, Timestamp, Topic,
    VersionedPayload, WorkflowName,
};

#[test]
fn persistence_values_promote_only_after_fallible_validation() {
    let usd = MoneyColumns::try_new(15_900, "usd").expect("USD money should decode");
    assert_eq!(usd.minor_units(), 15_900);
    assert_eq!(usd.currency(), CurrencyCode::Usd);

    assert!(MoneyColumns::try_new(-1, "usd").is_err());
    assert!(MoneyColumns::try_new(1, "US dollars").is_err());
    assert!(Timestamp::try_new("not-a-timestamp").is_err());
    assert!(Period::try_new("2026-07-01T00:00:00Z", "2026-06-01T00:00:00Z").is_err());
    assert!(Id::try_new("not-a-uuid").is_err());
    assert!(IdempotencyKey::try_new("   ").is_err());
    assert!(WorkflowName::try_new("Data Quality Hygiene").is_err());
    assert!(Topic::try_new("customer.messages.send").is_err());
}

#[test]
fn source_refs_and_versioned_payloads_reject_invalid_relationships() {
    let source_ref = SourceRefColumns::try_new(
        "gingr",
        "reservation",
        "reservation-42",
        "2026-06-18T14:00:00Z",
        "gingr-reservation.v1",
    )
    .expect("complete source lineage should decode");
    assert_eq!(source_ref.record_id().as_str(), "reservation-42");

    assert!(
        SourceRefColumns::try_new(
            "gingr",
            "reservation",
            "reservation-42",
            "not-a-timestamp",
            "gingr-reservation.v1",
        )
        .is_err()
    );
    assert!(
        SourceRefColumns::try_new(
            "gingr",
            "reservation",
            "reservation-42",
            "2026-06-18T14:00:00Z",
            "",
        )
        .is_err()
    );

    let payload = VersionedPayload::try_new(
        "workflow-event.v1",
        json!({"schema_version": "workflow-event.v1", "source_refs": []}),
    )
    .expect("matching version should decode");
    assert_eq!(payload.version().as_str(), "workflow-event.v1");

    assert!(
        VersionedPayload::try_new(
            "workflow-event.v1",
            json!({"schema_version": "workflow-event.v2", "source_refs": []}),
        )
        .is_err()
    );
    assert!(VersionedPayload::try_new("workflow-event.v1", json!({"source_refs": []}),).is_err());
    assert!(VersionedPayload::try_new("workflow-event.v1", json!([])).is_err());
}

#[test]
fn raw_sql_rows_remain_private_to_the_postgres_adapter() {
    let source = include_str!("../src/workflow_repository.rs");
    assert!(source.contains("struct SourceQualityBacklogRow"));
    assert!(!source.contains("pub struct SourceQualityBacklogRow"));
    assert!(source.contains("TryFrom<SourceQualityBacklogRow>"));
}
