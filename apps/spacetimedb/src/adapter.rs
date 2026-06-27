//! Trait implementations that adapt app ports to SpacetimeDB row collections.
//!
//! The adapter stores/retrieves rows only. It promotes rows into app/domain types
//! before app services evaluate authorization or capture outcomes.

use app::data_quality_hygiene as hygiene;

use crate::{
    authz,
    storage::review_queue::{
        BlockedActionAttemptRow, HygieneAuditEventRow, HygieneOutcomeRow, ReviewQueueItemRow, codec,
    },
    tables::{LocationScopeRow, RoleAssignmentRow, StaffActorRow},
};

/// Read-side actor directory backed by SpacetimeDB rows loaded at reducer entry.
pub struct ActorDirectoryAdapter {
    actors: Vec<StaffActorRow>,
    roles: Vec<RoleAssignmentRow>,
    scopes: Vec<LocationScopeRow>,
}

impl ActorDirectoryAdapter {
    /// Creates an actor directory from table snapshots.
    pub fn new(
        actors: Vec<StaffActorRow>,
        roles: Vec<RoleAssignmentRow>,
        scopes: Vec<LocationScopeRow>,
    ) -> Self {
        Self {
            actors,
            roles,
            scopes,
        }
    }
}

impl hygiene::ActorDirectory for ActorDirectoryAdapter {
    fn resolve_actor(&self, actor_id: &hygiene::ActorId) -> Option<hygiene::ActorAssignment> {
        let actor_id = actor_id.as_ref();
        let row = self.actors.iter().find(|row| row.actor_id == actor_id)?;
        authz::actor_assignment_from_rows(row, self.roles.iter(), self.scopes.iter())
    }
}

/// Review queue adapter backed by SpacetimeDB rows loaded at reducer entry.
pub struct ReviewQueueAdapter {
    items: Vec<ReviewQueueItemRow>,
}

impl ReviewQueueAdapter {
    /// Creates a review queue from table snapshots.
    pub fn new(items: Vec<ReviewQueueItemRow>) -> Self {
        Self { items }
    }
}

impl hygiene::ReviewQueueStore for ReviewQueueAdapter {
    fn review_item_for_action(
        &self,
        action_id: &hygiene::ActionId,
    ) -> Option<hygiene::ReviewQueueItem> {
        let row = self
            .items
            .iter()
            .find(|row| row.action_id == action_id.as_ref())?;
        codec::review_queue_item(row)
    }
}

/// Outcome recorder collecting storage rows for reducer persistence.
#[derive(Default)]
pub struct OutcomeRecorderAdapter {
    rows: Vec<HygieneOutcomeRow>,
}

impl OutcomeRecorderAdapter {
    /// Returns rows that should be persisted by the reducer.
    pub fn rows(&self) -> Vec<HygieneOutcomeRow> {
        self.rows.clone()
    }
}

impl hygiene::OutcomeRecorder for OutcomeRecorderAdapter {
    fn record_outcome(&mut self, outcome: hygiene::OutcomeRecord) -> hygiene::OutcomeReceipt {
        let receipt = hygiene::OutcomeReceipt::new(outcome.action_id().clone());
        self.rows.push(codec::hygiene_outcome_row(&outcome, 0));
        receipt
    }
}

/// Audit log collecting accepted-capture audit rows for reducer persistence.
#[derive(Default)]
pub struct AuditLogAdapter {
    rows: Vec<HygieneAuditEventRow>,
}

impl AuditLogAdapter {
    /// Returns rows that should be persisted by the reducer.
    pub fn rows(&self) -> Vec<HygieneAuditEventRow> {
        self.rows.clone()
    }
}

impl hygiene::AuditLog for AuditLogAdapter {
    fn append_audit_record(&mut self, record: hygiene::AuditRecord) {
        let actor = codec::actor_ref_label(record.actor());
        self.rows.push(HygieneAuditEventRow {
            id: 0,
            action_id: record.action_id().as_ref().to_owned(),
            actor_id: actor.clone(),
            actor,
            blocked_actions: format!("{:?}", record.blocked_actions()),
            created_at: 0,
            schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
        });
    }
}

/// Blocked-action log collecting fail-closed audit rows for reducer persistence.
#[derive(Default)]
pub struct BlockedActionLogAdapter {
    rows: Vec<BlockedActionAttemptRow>,
    review_items: Vec<ReviewQueueItemRow>,
}

impl BlockedActionLogAdapter {
    /// Creates a blocked-action log with review-queue context for location projection.
    pub fn new(review_items: Vec<ReviewQueueItemRow>) -> Self {
        Self {
            rows: Vec::new(),
            review_items,
        }
    }

    /// Returns rows that should be persisted by the reducer.
    pub fn rows(&self) -> Vec<BlockedActionAttemptRow> {
        self.rows.clone()
    }

    fn location_id_for_action(&self, action_id: &hygiene::ActionId) -> String {
        self.review_items
            .iter()
            .find(|row| row.action_id == action_id.as_ref())
            .map(|row| row.location_id.clone())
            .unwrap_or_else(|| "unknown".to_owned())
    }
}

impl hygiene::BlockedActionLog for BlockedActionLogAdapter {
    fn record_blocked_action(&mut self, record: hygiene::BlockedActionRecord) {
        self.rows.push(BlockedActionAttemptRow {
            id: 0,
            action_id: record.action_id().as_ref().to_owned(),
            actor_id: record.actor_id().as_ref().to_owned(),
            location_id: self.location_id_for_action(record.action_id()),
            attempted_side_effect: "record_reviewed_outcome".to_owned(),
            reason: codec::blocked_reason_column(record.reason()),
            created_at: 0,
            schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
        });
    }
}
