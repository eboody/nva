use tokio_postgres::{Client, Error, GenericClient, NoTls};
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

struct ReviewLineage {
    workflow_event_id: Uuid,
    review_packet_id: Uuid,
    approval_record_id: Uuid,
    location_id: Uuid,
    correlation_id: String,
}

async fn insert_review_lineage<C: GenericClient + Sync>(
    client: &C,
    reviewed_action_id: &str,
    approval_status: &str,
) -> Result<ReviewLineage, Error> {
    let workflow_event_id = Uuid::new_v4();
    let review_packet_id = Uuid::new_v4();
    let approval_record_id = Uuid::new_v4();
    let subject_id = Uuid::new_v4();
    let location_id = Uuid::new_v4();
    let correlation_id = format!("outcome-correlation-{reviewed_action_id}");

    client
        .execute(
            "INSERT INTO locations (id, brand, name, timezone)
             VALUES ($1, 'PetSuites', 'Outcome authority test', 'UTC')",
            &[&location_id],
        )
        .await?;

    client
        .execute(
            "INSERT INTO workflow_events
             (id, workflow_name, event_kind, subject_kind, subject_id, idempotency_key, payload)
             VALUES ($1, 'manager_daily_brief', 'outcome_capture', 'location', $2::uuid, $3,
                     jsonb_build_object(
                         'location_id', $2::text,
                         'operating_day', '2026-08-15',
                         'correlation_id', $4::text,
                         'source_refs', jsonb_build_array(jsonb_build_object(
                             'system', 'test',
                             'record_type', 'reservation',
                             'record_id', 'source-1',
                             'observed_at', '2026-08-15T00:00:00Z',
                             'adapter_version', 'outcome-authority-test-v1'
                         ))
                     ))",
            &[
                &workflow_event_id,
                &location_id,
                &format!("outcome-authority-{workflow_event_id}"),
                &correlation_id,
            ],
        )
        .await?;
    client
        .execute(
            "INSERT INTO review_packets
             (id, subject_kind, subject_id, gate, status, workflow_event_id,
              reviewed_action_id, created_by_actor_kind, created_by_actor_id)
             VALUES ($1, 'message', $2, 'manager_approval', 'approved', $3, $4,
                     'manager', 'reviewer-1')",
            &[
                &review_packet_id,
                &subject_id,
                &workflow_event_id,
                &reviewed_action_id,
            ],
        )
        .await?;
    client
        .execute(
            "INSERT INTO approval_records
             (id, target_kind, target_id, gate, status, requested_by_actor_kind,
              requested_by_actor_id, requested_at, decided_by_actor_kind,
              decided_by_actor_id, decided_by_actor_persona, decided_at, review_packet_id)
             VALUES ($1, 'message', $2, 'manager_approval', $3, 'agent', 'agent-1', now(),
                     'manager', 'reviewer-1', 'general_manager', now(), $4)",
            &[
                &approval_record_id,
                &subject_id,
                &approval_status,
                &review_packet_id,
            ],
        )
        .await?;

    Ok(ReviewLineage {
        workflow_event_id,
        review_packet_id,
        approval_record_id,
        location_id,
        correlation_id,
    })
}

async fn insert_manager_outcome<C: GenericClient + Sync>(
    client: &C,
    lineage: &ReviewLineage,
    action_id: &str,
) -> Result<u64, Error> {
    insert_manager_outcome_with_source_refs(
        client,
        lineage,
        action_id,
        r#"[{"system":"test","record_type":"reservation","record_id":"source-1","observed_at":"2026-08-15T00:00:00Z","adapter_version":"outcome-authority-test-v1"}]"#,
    )
    .await
}

async fn insert_manager_outcome_with_source_refs<C: GenericClient + Sync>(
    client: &C,
    lineage: &ReviewLineage,
    action_id: &str,
    source_refs: &str,
) -> Result<u64, Error> {
    client
        .execute(
            "INSERT INTO manager_daily_brief_outcomes
             (workflow_event_id, approval_record_id, action_id, outcome, actor_id,
              actor_persona, feedback, owner_persona, action_kind, before_minutes,
              actual_minutes, reported_estimated_minutes_difference, location_id, operating_day, source_refs,
              correlation_id)
             VALUES ($1, $2, $3, 'completed', 'reviewer-1', 'general_manager', '',
                     'general_manager', 'resolve_checkout_exception', 10, 5, 5,
                     $4, DATE '2026-08-15', ($5::text)::jsonb, $6)",
            &[
                &lineage.workflow_event_id,
                &lineage.approval_record_id,
                &action_id,
                &lineage.location_id,
                &source_refs,
                &lineage.correlation_id,
            ],
        )
        .await
}

#[tokio::test]
async fn outcome_rejects_approval_for_another_action_in_the_same_workflow() {
    let Some(mut client) = test_client().await else {
        return;
    };
    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-exact", "approved")
        .await
        .unwrap();

    let error = insert_manager_outcome(&transaction, &lineage, "action-other")
        .await
        .expect_err("same-workflow approval for another action must fail closed");
    assert!(error.as_db_error().is_some());
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn outcome_rejects_nonapproved_review_lineage() {
    let Some(mut client) = test_client().await else {
        return;
    };
    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-rejected", "rejected")
        .await
        .unwrap();

    let error = insert_manager_outcome(&transaction, &lineage, "action-rejected")
        .await
        .expect_err("rejected approval must not support reviewed outcome evidence");
    assert!(error.as_db_error().is_some());
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn outcome_rejects_malformed_nonempty_source_provenance() {
    let Some(mut client) = test_client().await else {
        return;
    };
    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-malformed-source", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE workflow_events
                SET payload = jsonb_set(payload, '{source_refs}', '[null]'::jsonb)
              WHERE id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .unwrap();

    insert_manager_outcome_with_source_refs(
        &transaction,
        &lineage,
        "action-malformed-source",
        "[null]",
    )
    .await
    .expect_err("non-empty malformed provenance must fail closed");
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn outcome_rejects_actor_kind_persona_and_scope_lineage_mismatches() {
    let Some(mut client) = test_client().await else {
        return;
    };
    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-wrong-actor-kind", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE approval_records SET decided_by_actor_kind = 'system' WHERE id = $1",
            &[&lineage.approval_record_id],
        )
        .await
        .expect_err("incoherent actor-kind/persona decision evidence must fail at admission");
    transaction.rollback().await.unwrap();

    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-wrong-persona", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE approval_records SET decided_by_actor_persona = 'front_desk_lead' WHERE id = $1",
            &[&lineage.approval_record_id],
        )
        .await
        .expect_err("incoherent persona/actor-kind decision evidence must fail at admission");
    transaction.rollback().await.unwrap();

    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-wrong-scope", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE workflow_events
                SET payload = jsonb_set(payload, '{correlation_id}', to_jsonb('other-correlation'::text))
              WHERE id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .unwrap();
    insert_manager_outcome(&transaction, &lineage, "action-wrong-scope")
        .await
        .expect_err("outcome correlation must match reviewed workflow scope");
    transaction.rollback().await.unwrap();

    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-wrong-subject", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE workflow_events SET subject_kind = 'pet', subject_id = gen_random_uuid() WHERE id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .unwrap();
    insert_manager_outcome(&transaction, &lineage, "action-wrong-subject")
        .await
        .expect_err("outcome location must match the relational workflow subject");
    transaction.rollback().await.unwrap();

    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-wrong-day", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE workflow_events SET payload = jsonb_set(payload, '{operating_day}', to_jsonb('2026-08-14'::text)) WHERE id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .unwrap();
    insert_manager_outcome(&transaction, &lineage, "action-wrong-day")
        .await
        .expect_err("outcome day must match the reviewed workflow scope");
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn approval_decision_chronology_rejects_decisions_before_requests() {
    let Some(mut client) = test_client().await else {
        return;
    };
    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-reversed-chronology", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE approval_records SET requested_at = decided_at + interval '1 minute' WHERE id = $1",
            &[&lineage.approval_record_id],
        )
        .await
        .expect_err("approval decision cannot precede its request");
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn accepted_outcome_freezes_approval_and_review_packet_lineage() {
    let Some(mut client) = test_client().await else {
        return;
    };
    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-immutable", "approved")
        .await
        .unwrap();
    insert_manager_outcome(&transaction, &lineage, "action-immutable")
        .await
        .unwrap();

    transaction
        .batch_execute("SAVEPOINT approval_mutation")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE approval_records SET decided_by_actor_id = 'other-reviewer' WHERE id = $1",
            &[&lineage.approval_record_id],
        )
        .await
        .expect_err("referenced approval authority must be immutable");
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT approval_mutation")
        .await
        .unwrap();

    transaction
        .batch_execute("SAVEPOINT packet_mutation")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE review_packets SET reviewed_action_id = 'other-action' WHERE id = $1",
            &[&lineage.review_packet_id],
        )
        .await
        .expect_err("referenced review packet lineage must be immutable");
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT packet_mutation")
        .await
        .unwrap();

    transaction
        .batch_execute("SAVEPOINT workflow_event_mutation")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE workflow_events SET payload = '{\"rewritten\":true}'::jsonb WHERE id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .expect_err("referenced workflow event lineage must be immutable");
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT workflow_event_mutation")
        .await
        .unwrap();

    transaction
        .batch_execute("SAVEPOINT outcome_mutation")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE manager_daily_brief_outcomes SET feedback = 'rewritten' WHERE workflow_event_id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .expect_err("accepted outcome evidence must be immutable");
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT outcome_mutation")
        .await
        .unwrap();

    transaction
        .batch_execute("SAVEPOINT outcome_deletion")
        .await
        .unwrap();
    transaction
        .execute(
            "DELETE FROM manager_daily_brief_outcomes WHERE workflow_event_id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .expect_err("accepted outcome evidence must not be deletable");
    transaction
        .batch_execute("ROLLBACK TO SAVEPOINT outcome_deletion")
        .await
        .unwrap();

    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn manager_outcome_rejects_data_quality_workflow_lineage_transplant() {
    let Some(mut client) = test_client().await else {
        return;
    };
    let transaction = client.transaction().await.unwrap();
    let lineage = insert_review_lineage(&transaction, "action-transplant", "approved")
        .await
        .unwrap();
    transaction
        .execute(
            "UPDATE workflow_events
                SET workflow_name = 'data_quality_hygiene', event_kind = 'context_created'
              WHERE id = $1",
            &[&lineage.workflow_event_id],
        )
        .await
        .unwrap();

    insert_manager_outcome(&transaction, &lineage, "action-transplant")
        .await
        .expect_err("data-quality lineage must not authorize a manager-brief outcome");
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn outcome_admission_serializes_with_concurrent_lineage_mutation() {
    let Some(mut setup_client) = test_client().await else {
        return;
    };
    let setup = setup_client.transaction().await.unwrap();
    let action_id = format!("action-concurrent-{}", Uuid::new_v4());
    let lineage = insert_review_lineage(&setup, &action_id, "approved")
        .await
        .unwrap();
    setup.commit().await.unwrap();

    let Some(mut outcome_client) = test_client().await else {
        return;
    };
    let Some(mut mutation_client) = test_client().await else {
        return;
    };
    let outcome_transaction = outcome_client.transaction().await.unwrap();
    insert_manager_outcome(&outcome_transaction, &lineage, &action_id)
        .await
        .unwrap();

    let approval_record_id = lineage.approval_record_id;
    let mut mutation = tokio::spawn(async move {
        let transaction = mutation_client.transaction().await.unwrap();
        let result = transaction
            .execute(
                "UPDATE approval_records
                    SET decided_by_actor_id = 'concurrent-rewriter'
                  WHERE id = $1",
                &[&approval_record_id],
            )
            .await;
        let _ = transaction.rollback().await;
        result
    });
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), &mut mutation)
            .await
            .is_err(),
        "concurrent mutation should block on the outcome admission lineage lock"
    );
    outcome_transaction.commit().await.unwrap();

    mutation
        .await
        .unwrap()
        .expect_err("concurrent lineage mutation must observe the committed outcome and fail");
}
