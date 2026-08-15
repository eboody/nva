//! Storage implementations of application-owned workflow repository ports.

use std::collections::BTreeMap;

use app::workflow_repository::source_quality_backlog;
use async_trait::async_trait;
use serde_json::Value;
use tokio_postgres::NoTls;

use crate::persistence::{Id, SourceRefColumns, Version};

#[derive(Debug)]
struct SourceQualityBacklogRow {
    issue_ref: String,
    location_id: Option<String>,
    tenant_id: Option<String>,
    affected_entity_kind: String,
    affected_entity_id: String,
    field_path: String,
    issue_kind: String,
    severity: String,
    freshness: String,
    sensitivity: String,
    workflow_blocking: String,
    owner_persona: String,
    review_gate: String,
    resolution_status: String,
    source_refs: Value,
    workflow_event_id: Option<String>,
    latest_outcome_id: Option<String>,
    projection_version: String,
    caveats: Vec<String>,
}

impl TryFrom<SourceQualityBacklogRow> for source_quality_backlog::Item {
    type Error = crate::persistence::Error;

    fn try_from(row: SourceQualityBacklogRow) -> Result<Self, Self::Error> {
        if let Some(id) = &row.location_id {
            Id::try_new(id)?;
        }
        if let Some(id) = &row.workflow_event_id {
            Id::try_new(id)?;
        }
        if let Some(id) = &row.latest_outcome_id {
            Id::try_new(id)?;
        }
        Version::try_new(row.projection_version.clone())?;
        validate_source_refs(&row.source_refs)?;

        Ok(Self {
            issue_ref: row.issue_ref,
            location_id: row.location_id,
            tenant_id: row.tenant_id,
            affected_entity_kind: row.affected_entity_kind,
            affected_entity_id: row.affected_entity_id,
            field_path: row.field_path,
            issue_kind: row.issue_kind,
            severity: row.severity,
            freshness: row.freshness,
            sensitivity: row.sensitivity,
            workflow_blocking: row.workflow_blocking,
            owner_persona: row.owner_persona,
            review_gate: row.review_gate,
            resolution_status: row.resolution_status,
            source_refs: row.source_refs,
            workflow_event_id: row.workflow_event_id,
            latest_outcome_id: row.latest_outcome_id,
            projection_version: row.projection_version,
            caveats: row.caveats,
        })
    }
}

fn validate_source_refs(value: &Value) -> crate::persistence::Result<()> {
    let refs = value
        .as_array()
        .ok_or(crate::persistence::Error::InvalidCode {
            field: "source_refs",
        })?;
    for source_ref in refs {
        let field = |name| {
            source_ref.get(name).and_then(Value::as_str).ok_or(
                crate::persistence::Error::InvalidCode {
                    field: "source_refs",
                },
            )
        };
        SourceRefColumns::try_new(
            field("system")?,
            field("record_type")?,
            field("record_id")?,
            field("observed_at")?,
            field("adapter_version")?,
        )?;
    }
    Ok(())
}

/// Deterministic storage adapter for app-owned outcome repository ports.
#[derive(Clone)]
pub struct InMemoryOutcomes<T> {
    outcomes: Vec<T>,
    idempotency: BTreeMap<IdempotencyKey, OperationFingerprint>,
}

impl<T> Default for InMemoryOutcomes<T> {
    fn default() -> Self {
        Self {
            outcomes: Vec::new(),
            idempotency: BTreeMap::new(),
        }
    }
}

/// Validated client operation key used to make one outcome command replay-safe.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Promotes a non-empty boundary value into a storage idempotency identity.
    pub fn try_new(raw: impl Into<String>) -> Result<Self, IdempotencyValueError> {
        let raw = raw.into();
        (!raw.is_empty())
            .then_some(Self(raw))
            .ok_or(IdempotencyValueError::Empty)
    }
}

/// Semantic fingerprint of every command fact that can change an outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationFingerprint(String);

impl OperationFingerprint {
    /// Promotes a non-empty deterministic digest into a semantic fingerprint.
    pub fn try_new(raw: impl Into<String>) -> Result<Self, IdempotencyValueError> {
        let raw = raw.into();
        (!raw.is_empty())
            .then_some(Self(raw))
            .ok_or(IdempotencyValueError::Empty)
    }
}

/// Invalid storage idempotency boundary values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IdempotencyValueError {
    /// The boundary supplied no stable identity or fingerprint bytes.
    #[error("idempotency value must not be empty")]
    Empty,
}

/// Atomic outcome of checking and recording an idempotent operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotentRecord {
    /// A new semantic operation was recorded exactly once.
    Recorded {
        /// Number of outcomes retained after recording.
        retained_count: usize,
    },
    /// The key and semantic fingerprint matched an already-recorded operation.
    Replay {
        /// Number of outcomes retained; replay never increments it.
        retained_count: usize,
    },
    /// The key existed with a different semantic fingerprint.
    Conflict,
}

impl<T> InMemoryOutcomes<T> {
    /// Atomically checks the key/fingerprint pair and records only a new operation.
    ///
    /// The caller must hold exclusive access to this repository value for the
    /// duration of the call; `&mut self` makes that requirement explicit.
    pub fn record_idempotently(
        &mut self,
        key: IdempotencyKey,
        fingerprint: OperationFingerprint,
        outcome: T,
    ) -> IdempotentRecord {
        match self.idempotency.get(&key) {
            Some(existing) if existing == &fingerprint => IdempotentRecord::Replay {
                retained_count: self.outcomes.len(),
            },
            Some(_) => IdempotentRecord::Conflict,
            None => {
                self.idempotency.insert(key, fingerprint);
                self.outcomes.push(outcome);
                IdempotentRecord::Recorded {
                    retained_count: self.outcomes.len(),
                }
            }
        }
    }
}

impl<T> app::workflow_repository::OutcomeRepository for InMemoryOutcomes<T> {
    type Outcome = T;

    fn record(&mut self, outcome: Self::Outcome) -> usize {
        self.outcomes.push(outcome);
        self.outcomes.len()
    }

    fn outcomes(&self) -> &[Self::Outcome] {
        &self.outcomes
    }
}

#[cfg(test)]
mod tests {
    use app::workflow_repository::OutcomeRepository;

    use super::*;

    #[test]
    fn idempotent_outcomes_replay_equal_fingerprints_and_reject_drift_atomically() {
        let mut outcomes = InMemoryOutcomes::default();
        let key = IdempotencyKey::try_new("manager-brief-1").unwrap();
        let fingerprint = OperationFingerprint::try_new("sha256:first").unwrap();

        assert_eq!(
            outcomes.record_idempotently(key.clone(), fingerprint.clone(), "record"),
            IdempotentRecord::Recorded { retained_count: 1 }
        );
        assert_eq!(
            outcomes.record_idempotently(key.clone(), fingerprint, "ignored replay"),
            IdempotentRecord::Replay { retained_count: 1 }
        );
        assert_eq!(
            outcomes.record_idempotently(
                key,
                OperationFingerprint::try_new("sha256:drift").unwrap(),
                "ignored conflict",
            ),
            IdempotentRecord::Conflict
        );
        assert_eq!(outcomes.outcomes(), &["record"]);
    }
}

/// Postgres adapter for the application-owned source-quality backlog read model.
#[derive(Clone)]
pub struct PostgresSourceQualityBacklog {
    database_url: String,
}

impl core::fmt::Debug for PostgresSourceQualityBacklog {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PostgresSourceQualityBacklog")
            .field("database_url", &"<redacted>")
            .finish()
    }
}

impl PostgresSourceQualityBacklog {
    /// Creates an inert adapter configuration; no connection occurs until queried.
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
        }
    }
}

#[async_trait]
impl source_quality_backlog::Repository for PostgresSourceQualityBacklog {
    async fn prioritized_items_for_location(
        &self,
        location_id: domain::entities::LocationId,
    ) -> Result<Vec<source_quality_backlog::Item>, source_quality_backlog::Error> {
        let (client, connection) = tokio_postgres::connect(&self.database_url, NoTls)
            .await
            .map_err(|_| source_quality_backlog::Error::Unavailable)?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                tracing::warn!(safe_error_class = "database_connection", %error, "postgres read-model connection ended");
            }
        });

        let rows = client
            .query(
                "SELECT issue_ref,
                        location_id::text AS location_id,
                        tenant_id,
                        affected_entity_kind,
                        affected_entity_id,
                        field_path,
                        issue_kind,
                        severity,
                        freshness,
                        sensitivity,
                        workflow_blocking,
                        owner_persona,
                        review_gate,
                        resolution_status,
                        source_refs,
                        workflow_event_id::text AS workflow_event_id,
                        latest_outcome_id::text AS latest_outcome_id,
                        projection_version,
                        caveats
                 FROM source_quality_backlog
                 WHERE location_id = $1
                 ORDER BY CASE severity
                        WHEN 'critical' THEN 1
                        WHEN 'high' THEN 2
                        WHEN 'medium' THEN 3
                        ELSE 4
                    END,
                    issue_ref
                 LIMIT 50",
                &[&location_id.get()],
            )
            .await
            .map_err(|_| source_quality_backlog::Error::Unavailable)?;

        rows.into_iter()
            .map(|row| SourceQualityBacklogRow {
                issue_ref: row.get("issue_ref"),
                location_id: row.get("location_id"),
                tenant_id: row.get("tenant_id"),
                affected_entity_kind: row.get("affected_entity_kind"),
                affected_entity_id: row.get("affected_entity_id"),
                field_path: row.get("field_path"),
                issue_kind: row.get("issue_kind"),
                severity: row.get("severity"),
                freshness: row.get("freshness"),
                sensitivity: row.get("sensitivity"),
                workflow_blocking: row.get("workflow_blocking"),
                owner_persona: row.get("owner_persona"),
                review_gate: row.get("review_gate"),
                resolution_status: row.get("resolution_status"),
                source_refs: row.get("source_refs"),
                workflow_event_id: row.get("workflow_event_id"),
                latest_outcome_id: row.get("latest_outcome_id"),
                projection_version: row.get("projection_version"),
                caveats: row.get("caveats"),
            })
            .map(source_quality_backlog::Item::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| source_quality_backlog::Error::Unavailable)
    }
}
