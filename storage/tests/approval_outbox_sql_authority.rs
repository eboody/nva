use serde_json::json;
use tokio_postgres::{Client, Error, GenericClient, NoTls, error::SqlState};
use uuid::Uuid;

async fn test_client() -> Option<Client> {
    let database_url = std::env::var("TEST_DATABASE_URL").ok()?;
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
        .await
        .expect("TEST_DATABASE_URL must accept connections");
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            panic!("postgres test connection failed: {error}");
        }
    });
    Some(client)
}

async fn insert_approval<C: GenericClient + Sync>(
    client: &C,
    status: &str,
    target_id: Uuid,
) -> Result<Uuid, Error> {
    let approval_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO approval_records (
                id, target_kind, target_id, gate, status,
                requested_by_actor_kind, requested_by_actor_id, requested_at,
                decided_by_actor_kind, decided_by_actor_id, decided_at
             ) VALUES ($1, 'message', $2, 'manager_approval', $3,
                'agent', 'agent:test', NOW(),
                CASE WHEN $3 = 'approved' THEN 'manager' END,
                CASE WHEN $3 = 'approved' THEN 'manager:test' END,
                CASE WHEN $3 = 'approved' THEN NOW() END)",
            &[&approval_id, &target_id, &status],
        )
        .await?;
    Ok(approval_id)
}

async fn insert_binding<C: GenericClient + Sync>(
    client: &C,
    approval_id: Uuid,
    payload: &serde_json::Value,
) -> Result<(), Error> {
    client
        .execute(
            "INSERT INTO approval_outbox_bindings (
                approval_record_id, topic, review_gate, aggregate_kind, aggregate_id, payload,
                authorized_by_actor_kind, authorized_by_actor_id, authorized_at
             ) SELECT id, 'internal.data_quality_hygiene.reviewed_handoff', gate,
                target_kind, target_id, $2,
                COALESCE(decided_by_actor_kind, 'manager'),
                COALESCE(decided_by_actor_id, 'manager:test'),
                COALESCE(decided_at, NOW())
               FROM approval_records WHERE id = $1",
            &[&approval_id, payload],
        )
        .await?;
    Ok(())
}

#[tokio::test]
async fn database_consumes_one_matching_internal_handoff_binding_per_approval() {
    let Some(mut client) = test_client().await else {
        eprintln!("TEST_DATABASE_URL unset; skipping live PostgreSQL authority contract");
        return;
    };
    let transaction = client.transaction().await.expect("transaction starts");
    let target_id = Uuid::new_v4();
    let approval_id = insert_approval(&transaction, "approved", target_id)
        .await
        .expect("approved record inserts");
    let payload = json!({"kind": "data_quality_hygiene_handoff", "target_id": target_id});

    insert_binding(&transaction, approval_id, &payload)
        .await
        .expect("approved review issues a durable binding");

    let outbox_id = Uuid::new_v4();
    transaction
        .execute(
            "INSERT INTO outbox_records (
                id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
                approval_record_id, review_gate, status
             ) VALUES ($1, 'test:outbox:primary', 'message', $2,
                'internal.data_quality_hygiene.reviewed_handoff', $3, $4,
                'manager_approval', 'pending')",
            &[&outbox_id, &target_id, &payload, &approval_id],
        )
        .await
        .expect("matching binding authorizes one outbox insertion");

    let consumed_by: Uuid = transaction
        .query_one(
            "SELECT consumed_by_outbox_id FROM approval_outbox_bindings WHERE approval_record_id = $1",
            &[&approval_id],
        )
        .await
        .expect("binding remains inspectable")
        .get(0);
    assert_eq!(consumed_by, outbox_id);

    let duplicate = transaction
        .execute(
            "INSERT INTO outbox_records (
                id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
                approval_record_id, review_gate, status
             ) VALUES ($1, 'test:outbox:duplicate', 'message', $2,
                'internal.data_quality_hygiene.reviewed_handoff', $3, $4,
                'manager_approval', 'pending')",
            &[&Uuid::new_v4(), &target_id, &payload, &approval_id],
        )
        .await
        .expect_err("a consumed approval cannot authorize another outbox row");
    let duplicate_db_error = duplicate
        .as_db_error()
        .expect("database rejection has detail");
    assert!(
        duplicate_db_error.code() == &SqlState::UNIQUE_VIOLATION
            || duplicate_db_error
                .message()
                .contains("matching unconsumed approval_outbox_binding")
    );

    transaction.rollback().await.expect("test rolls back");
}

#[tokio::test]
async fn database_rejects_unapproved_mismatched_or_unbound_handoffs() {
    let Some(mut client) = test_client().await else {
        eprintln!("TEST_DATABASE_URL unset; skipping live PostgreSQL authority contract");
        return;
    };
    let transaction = client.transaction().await.expect("transaction starts");
    let target_id = Uuid::new_v4();
    let pending_approval = insert_approval(&transaction, "approval_requested", target_id)
        .await
        .expect("pending review inserts");
    let payload = json!({"kind": "data_quality_hygiene_handoff", "target_id": target_id});

    transaction
        .batch_execute("SAVEPOINT pending_binding_case")
        .await
        .expect("savepoint starts");
    let pending_binding = insert_binding(&transaction, pending_approval, &payload)
        .await
        .expect_err("pending evidence cannot issue authority");
    assert!(
        pending_binding
            .as_db_error()
            .expect("database rejection has detail")
            .message()
            .contains("matching approved decision authority")
    );
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT pending_binding_case")
        .await
        .expect("failed statement is isolated");

    let approval_id = insert_approval(&transaction, "approved", target_id)
        .await
        .expect("approved record inserts");
    insert_binding(&transaction, approval_id, &payload)
        .await
        .expect("matching binding inserts");

    transaction
        .batch_execute("SAVEPOINT mismatched_outbox_case")
        .await
        .expect("savepoint starts");
    let mismatched = transaction
        .execute(
            "INSERT INTO outbox_records (
                id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
                approval_record_id, review_gate, status
             ) VALUES ($1, 'test:outbox:mismatch', 'message', $2,
                'internal.site_finance.reviewed_handoff', $3, $4,
                'manager_approval', 'pending')",
            &[&Uuid::new_v4(), &target_id, &payload, &approval_id],
        )
        .await
        .expect_err("binding topic cannot drift at SQL boundary");
    assert!(
        mismatched
            .as_db_error()
            .expect("database rejection has detail")
            .message()
            .contains("matching unconsumed approval_outbox_binding")
    );
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT mismatched_outbox_case")
        .await
        .expect("failed statement is isolated");

    let unbound_approval = insert_approval(&transaction, "approved", Uuid::new_v4())
        .await
        .expect("second approval inserts");
    transaction
        .batch_execute("SAVEPOINT unbound_outbox_case")
        .await
        .expect("savepoint starts");
    let unbound = transaction
        .execute(
            "INSERT INTO outbox_records (
                id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
                approval_record_id, review_gate, status
             ) SELECT $1, 'test:outbox:unbound', target_kind, target_id,
                'internal.data_quality_hygiene.reviewed_handoff', $2, id, gate, 'pending'
               FROM approval_records WHERE id = $3",
            &[&Uuid::new_v4(), &payload, &unbound_approval],
        )
        .await
        .expect_err("approved evidence without a binding is not authority");
    assert!(
        unbound
            .as_db_error()
            .expect("database rejection has detail")
            .message()
            .contains("matching unconsumed approval_outbox_binding")
    );
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT unbound_outbox_case")
        .await
        .expect("failed statement is isolated");

    transaction.rollback().await.expect("test rolls back");
}

#[tokio::test]
async fn database_rejects_authority_identity_updates_and_deletes() {
    let Some(mut client) = test_client().await else {
        eprintln!("TEST_DATABASE_URL unset; skipping live PostgreSQL immutability contract");
        return;
    };
    let transaction = client.transaction().await.expect("transaction starts");
    let target_id = Uuid::new_v4();
    let approval_id = insert_approval(&transaction, "approved", target_id)
        .await
        .expect("approved record inserts");
    let payload = json!({"kind": "data_quality_hygiene_handoff", "target_id": target_id});
    insert_binding(&transaction, approval_id, &payload)
        .await
        .expect("binding inserts");
    let outbox_id = Uuid::new_v4();
    transaction
        .execute(
            "INSERT INTO outbox_records (
                id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
                approval_record_id, review_gate, status
             ) VALUES ($1, 'test:outbox:immutable', 'message', $2,
                'internal.data_quality_hygiene.reviewed_handoff', $3, $4,
                'manager_approval', 'pending')",
            &[&outbox_id, &target_id, &payload, &approval_id],
        )
        .await
        .expect("matching outbox inserts");

    for (savepoint, statement, expected) in [
        (
            "outbox_update_case",
            "UPDATE outbox_records SET topic = 'internal.site_finance.reviewed_handoff' WHERE id = $1",
            "outbox authority identity is immutable",
        ),
        (
            "outbox_delete_case",
            "DELETE FROM outbox_records WHERE id = $1",
            "outbox_records are durable history",
        ),
        (
            "binding_update_case",
            "UPDATE approval_outbox_bindings SET payload = '{\"forged\":true}'::jsonb WHERE consumed_by_outbox_id = $1",
            "approval_outbox_bindings permit only one relationally proven consumption transition",
        ),
        (
            "binding_delete_case",
            "DELETE FROM approval_outbox_bindings WHERE consumed_by_outbox_id = $1",
            "approval_outbox_bindings are immutable",
        ),
    ] {
        transaction
            .batch_execute(&format!("SAVEPOINT {savepoint}"))
            .await
            .expect("savepoint starts");
        let error = transaction
            .execute(statement, &[&outbox_id])
            .await
            .expect_err("authority history mutation is rejected");
        assert!(
            error
                .as_db_error()
                .expect("database rejection has detail")
                .message()
                .contains(expected)
        );
        transaction
            .batch_execute(&format!("ROLLBACK TO SAVEPOINT {savepoint}"))
            .await
            .expect("failed statement is isolated");
    }

    transaction
        .batch_execute("SAVEPOINT approval_decision_update_case")
        .await
        .expect("savepoint starts");
    let approval_update = transaction
        .execute(
            "UPDATE approval_records SET decided_by_actor_id = 'forged-manager' WHERE id = $1",
            &[&approval_id],
        )
        .await
        .expect_err("decision evidence cannot drift after binding");
    assert!(
        approval_update
            .as_db_error()
            .expect("database rejection has detail")
            .message()
            .contains("cannot change approval after")
    );
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT approval_decision_update_case")
        .await
        .expect("failed statement is isolated");

    transaction.rollback().await.expect("test rolls back");
}

#[tokio::test]
async fn nested_trigger_depth_cannot_forge_binding_consumption() {
    let Some(mut client) = test_client().await else {
        eprintln!("TEST_DATABASE_URL unset; skipping live PostgreSQL nested-trigger contract");
        return;
    };
    let transaction = client.transaction().await.expect("transaction starts");
    let target_id = Uuid::new_v4();
    let approval_id = insert_approval(&transaction, "approved", target_id)
        .await
        .expect("approved record inserts");
    let payload = json!({"kind": "data_quality_hygiene_handoff", "target_id": target_id});
    insert_binding(&transaction, approval_id, &payload)
        .await
        .expect("binding inserts");

    transaction
        .batch_execute(
            "CREATE TEMP TABLE forged_outbox_trigger_source (
                id uuid NOT NULL,
                approval_record_id uuid NOT NULL,
                topic text NOT NULL,
                review_gate text NOT NULL,
                aggregate_kind text NOT NULL,
                aggregate_id uuid NOT NULL,
                payload jsonb NOT NULL
             );
             CREATE TRIGGER forged_nested_consumption
             AFTER INSERT ON forged_outbox_trigger_source
             FOR EACH ROW EXECUTE FUNCTION enforce_outbox_internal_handoff_binding();
             SAVEPOINT forged_nested_trigger_case;",
        )
        .await
        .expect("adversarial trigger fixture installs");

    let forged_outbox_id = Uuid::new_v4();
    let error = transaction
        .execute(
            "INSERT INTO forged_outbox_trigger_source (
                id, approval_record_id, topic, review_gate, aggregate_kind, aggregate_id, payload
             ) VALUES ($1, $2, 'internal.data_quality_hygiene.reviewed_handoff',
                'manager_approval', 'message', $3, $4)",
            &[&forged_outbox_id, &approval_id, &target_id, &payload],
        )
        .await
        .expect_err("nested trigger depth is not consumption authority");
    assert!(
        error
            .as_db_error()
            .expect("database rejection has detail")
            .message()
            .contains("relationally proven consumption transition")
    );
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT forged_nested_trigger_case")
        .await
        .expect("failed adversarial statement is isolated");

    let consumed_by: Option<Uuid> = transaction
        .query_one(
            "SELECT consumed_by_outbox_id FROM approval_outbox_bindings WHERE approval_record_id = $1",
            &[&approval_id],
        )
        .await
        .expect("binding remains inspectable")
        .get(0);
    assert_eq!(consumed_by, None);
    transaction.rollback().await.expect("test rolls back");
}

#[tokio::test]
async fn binding_issuance_serializes_against_concurrent_approval_drift() {
    let Some(mut binding_client) = test_client().await else {
        eprintln!("TEST_DATABASE_URL unset; skipping live PostgreSQL approval race contract");
        return;
    };
    let Some(approval_client) = test_client().await else {
        unreachable!("the same configured PostgreSQL endpoint just accepted a connection");
    };

    let target_id = Uuid::new_v4();
    let approval_id = insert_approval(&binding_client, "approved", target_id)
        .await
        .expect("approved record inserts");
    let payload = json!({"kind": "data_quality_hygiene_handoff", "target_id": target_id});
    let transaction = binding_client
        .transaction()
        .await
        .expect("binding writer starts a transaction");
    insert_binding(&transaction, approval_id, &payload)
        .await
        .expect("binding writer locks and validates approval");

    let approval_update = tokio::spawn(async move {
        approval_client
            .execute(
                "UPDATE approval_records SET decided_by_actor_id = 'concurrent-forged-manager' WHERE id = $1",
                &[&approval_id],
            )
            .await
    });
    transaction
        .commit()
        .await
        .expect("binding writer commits the authority relation");

    let update_error = approval_update
        .await
        .expect("approval writer task completes")
        .expect_err("approval drift cannot race a committed binding");
    assert!(
        update_error
            .as_db_error()
            .expect("database rejection has structured detail")
            .message()
            .contains("cannot change approval after")
    );
}

#[tokio::test]
async fn concurrent_writers_cannot_consume_one_binding_twice() {
    let Some(mut first_client) = test_client().await else {
        eprintln!("TEST_DATABASE_URL unset; skipping live PostgreSQL concurrency contract");
        return;
    };
    let Some(second_client) = test_client().await else {
        unreachable!("the same configured PostgreSQL endpoint just accepted a connection");
    };

    let target_id = Uuid::new_v4();
    let approval_id = insert_approval(&first_client, "approved", target_id)
        .await
        .expect("approved record inserts");
    let payload = json!({"kind": "data_quality_hygiene_handoff", "target_id": target_id});
    insert_binding(&first_client, approval_id, &payload)
        .await
        .expect("binding inserts");

    let first_outbox_id = Uuid::new_v4();
    let second_outbox_id = Uuid::new_v4();
    let first_idempotency_key = format!("test:outbox:concurrent:first:{approval_id}");
    let second_idempotency_key = format!("test:outbox:concurrent:second:{approval_id}");
    let transaction = first_client
        .transaction()
        .await
        .expect("first writer starts a transaction");
    transaction
        .execute(
            "INSERT INTO outbox_records (
                id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
                approval_record_id, review_gate, status
             ) VALUES ($1, $5, 'message', $2,
                'internal.data_quality_hygiene.reviewed_handoff', $3, $4,
                'manager_approval', 'pending')",
            &[
                &first_outbox_id,
                &target_id,
                &payload,
                &approval_id,
                &first_idempotency_key,
            ],
        )
        .await
        .expect("first writer acquires the one binding");

    let second_payload = payload.clone();
    let second_insert = tokio::spawn(async move {
        second_client
            .execute(
                "INSERT INTO outbox_records (
                    id, idempotency_key, aggregate_kind, aggregate_id, topic, payload,
                    approval_record_id, review_gate, status
                 ) VALUES ($1, $5, 'message', $2,
                    'internal.data_quality_hygiene.reviewed_handoff', $3, $4,
                    'manager_approval', 'pending')",
                &[
                    &second_outbox_id,
                    &target_id,
                    &second_payload,
                    &approval_id,
                    &second_idempotency_key,
                ],
            )
            .await
    });

    transaction
        .commit()
        .await
        .expect("first writer commits its consumption");
    let second_error = second_insert
        .await
        .expect("second writer task completes")
        .expect_err("the competing writer cannot consume the committed binding");
    let db_error = second_error
        .as_db_error()
        .expect("database rejection has structured detail");
    assert!(
        db_error.code() == &SqlState::UNIQUE_VIOLATION
            || db_error
                .message()
                .contains("matching unconsumed approval_outbox_binding")
    );

    let consumed_by: Uuid = first_client
        .query_one(
            "SELECT consumed_by_outbox_id FROM approval_outbox_bindings WHERE approval_record_id = $1",
            &[&approval_id],
        )
        .await
        .expect("binding remains inspectable")
        .get(0);
    assert_eq!(consumed_by, first_outbox_id);
}
