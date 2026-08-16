//! Application-owned repository contracts shared by HTTP, storage, and worker adapters.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Safe aggregate counters exposed by a workflow repository without leaking row vocabulary.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCounters {
    /// Accepted inquiry workflow records.
    pub inquiry_count: usize,
    /// Review packets awaiting or retaining a decision.
    pub review_packet_count: usize,
    /// Append-only audit facts.
    pub audit_event_count: usize,
    /// Caller-reported workflow outcomes retained by the repository.
    pub outcome_count: usize,
    /// Internal outbox candidates; these are not live sends or provider writes.
    pub internal_outbox_candidate_count: usize,
    /// Internal candidates whose payload explicitly keeps live delivery disabled.
    pub review_gated_internal_outbox_count: usize,
}

/// Minimal application repository authority required by runtime health and metrics surfaces.
pub trait Repository {
    /// Returns aggregate workflow facts without exposing persistence records.
    fn runtime_counters(&self) -> RuntimeCounters;
}

/// App-owned persistence capability for one workflow's caller-reported outcomes.
///
/// The associated value remains workflow-specific; this port owns record/list semantics
/// without forcing application code to depend on a storage row representation.
pub trait OutcomeRepository {
    /// Reported-outcome value accepted by this workflow repository.
    type Outcome;

    /// Records one reported outcome and returns the number retained by this adapter.
    fn record(&mut self, outcome: Self::Outcome) -> usize;

    /// Returns caller-reported outcomes for app-owned reporting use cases.
    fn outcomes(&self) -> &[Self::Outcome];
}

/// Source-quality backlog read model owned by the application contract.
pub mod source_quality_backlog {
    use super::*;
    use domain::entities;

    /// One source-quality issue projected for internal operational review.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Item {
        /// Stores the issue ref component of this boundary value.
        pub issue_ref: String,
        /// Stores the location id component of this boundary value.
        pub location_id: Option<String>,
        /// Stores the tenant id component of this boundary value.
        pub tenant_id: Option<String>,
        /// Stores the affected entity kind component of this boundary value.
        pub affected_entity_kind: String,
        /// Stores the affected entity id component of this boundary value.
        pub affected_entity_id: String,
        /// Stores the field path component of this boundary value.
        pub field_path: String,
        /// Stores the issue kind component of this boundary value.
        pub issue_kind: String,
        /// Stores the severity component of this boundary value.
        pub severity: String,
        /// Stores the freshness component of this boundary value.
        pub freshness: String,
        /// Stores the sensitivity component of this boundary value.
        pub sensitivity: String,
        /// Stores the workflow blocking component of this boundary value.
        pub workflow_blocking: String,
        /// Stores the owner persona component of this boundary value.
        pub owner_persona: String,
        /// Stores the review gate component of this boundary value.
        pub review_gate: String,
        /// Stores the resolution status component of this boundary value.
        pub resolution_status: String,
        /// Stores the source refs component of this boundary value.
        pub source_refs: Value,
        /// Stores the workflow event id component of this boundary value.
        pub workflow_event_id: Option<String>,
        /// Stores the latest outcome id component of this boundary value.
        pub latest_outcome_id: Option<String>,
        /// Stores the projection version component of this boundary value.
        pub projection_version: String,
        /// Stores the caveats component of this boundary value.
        pub caveats: Vec<String>,
    }

    /// Typed failures from a source-quality backlog adapter.
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        /// The configured adapter could not connect or query its backing store.
        #[error("source-quality backlog is unavailable")]
        Unavailable,
    }

    /// Application-owned query port for the source-quality backlog.
    #[async_trait]
    pub trait Repository {
        /// Returns the bounded, priority-ordered backlog visible to one location.
        async fn prioritized_items_for_location(
            &self,
            location_id: entities::LocationId,
        ) -> Result<Vec<Item>, Error>;
    }
}
