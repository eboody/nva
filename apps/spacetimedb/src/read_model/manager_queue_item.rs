//! Manager-facing public subscription rows for the hygiene review queue.
//!
//! Manager read models are separate from staff read models so subscription shape
//! can evolve independently from private storage rows and staff dashboard needs.

#![allow(
    missing_docs,
    reason = "SpacetimeDB table macros generate public accessors/traits without rustdoc hooks; row structs and fields remain documented."
)]

/// Public queue item for manager dashboards subscribed to approval-gated work.
#[spacetimedb::table(accessor = manager_queue_item, public)]
#[derive(Clone, Debug)]
pub struct ManagerQueueItemRow {
    /// Action id shown to dashboard clients.
    #[primary_key]
    pub action_id: String,
    /// Location id used by client subscription filters.
    #[index(btree)]
    pub location_id: String,
    /// Actor currently associated with the work, if any.
    #[index(btree)]
    pub actor_id: Option<String>,
    /// Actor that currently owns the work, if any.
    #[index(btree)]
    pub claimed_by_actor_id: Option<String>,
    /// Whether the item is currently waiting on manager approval.
    #[index(btree)]
    pub requires_manager_approval: bool,
    /// Queue status label for display.
    pub status_label: String,
    /// Optional source ref for traceability/filtering.
    #[index(btree)]
    pub source_ref_id: Option<String>,
    /// Data-quality issue ref for traceability.
    pub issue_ref: String,
    /// Staff recommendation routed for manager review.
    pub recommendation: Option<String>,
    /// Staff disposition routed for manager review.
    pub staff_disposition: Option<String>,
    /// Manager outcome once disposition is recorded.
    pub manager_outcome: Option<String>,
    /// Unix timestamp when the work entered the queue.
    #[index(btree)]
    pub created_at: u64,
    /// Unix timestamp when this read model last changed.
    pub updated_at: u64,
    /// Schema version for additive read-model evolution.
    pub schema_version: u32,
}
