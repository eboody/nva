//! Staff-facing public subscription rows for the hygiene review queue.
//!
//! These rows are client contracts for dashboards. They are intentionally
//! denormalized and are projected from storage/app facts rather than treated as
//! canonical domain objects.

#![allow(
    missing_docs,
    reason = "SpacetimeDB table macros generate public accessors/traits without rustdoc hooks; row structs and fields remain documented."
)]

/// Public queue item for staff dashboards subscribed to pending hygiene work.
#[spacetimedb::table(accessor = staff_queue_item, public)]
#[derive(Clone, Debug)]
pub struct StaffQueueItemRow {
    /// Action id shown to dashboard clients.
    #[primary_key]
    pub action_id: String,
    /// Location id used by client subscription filters.
    #[index(btree)]
    pub location_id: String,
    /// Optional actor assignment shown to the staff UI.
    #[index(btree)]
    pub actor_id: Option<String>,
    /// Actor that currently owns the work, if any.
    #[index(btree)]
    pub claimed_by_actor_id: Option<String>,
    /// Queue status label for display.
    pub status_label: String,
    /// Optional source ref for traceability/filtering.
    #[index(btree)]
    pub source_ref_id: Option<String>,
    /// Data-quality issue ref for traceability.
    pub issue_ref: String,
    /// Recommendation draft visible to subscribed staff dashboard clients.
    pub recommendation: Option<String>,
    /// Unix timestamp when the work entered the queue.
    #[index(btree)]
    pub created_at: u64,
    /// Unix timestamp when this read model last changed.
    pub updated_at: u64,
    /// Schema version for additive read-model evolution.
    pub schema_version: u32,
}

/// Public notice for denied actions and blocked live side-effect attempts.
#[spacetimedb::table(accessor = blocked_action_notice, public)]
#[derive(Clone, Debug)]
pub struct BlockedActionNoticeRow {
    /// Synthetic notice id.
    #[primary_key]
    pub id: u64,
    /// Action id for the blocked workflow attempt.
    #[index(btree)]
    pub action_id: String,
    /// Actor id that attempted the blocked side effect.
    #[index(btree)]
    pub actor_id: String,
    /// Location id associated with the blocked attempt.
    #[index(btree)]
    pub location_id: String,
    /// Side effect that remains blocked by the app/runtime boundary.
    pub attempted_side_effect: String,
    /// Display-safe reason label for subscribed dashboard clients.
    pub reason_label: String,
    /// Unix timestamp when the notice was projected.
    pub created_at: u64,
    /// Schema version for additive read-model evolution.
    pub schema_version: u32,
}

/// Public card for staff dashboards subscribed to reviewed hygiene outcomes.
#[spacetimedb::table(accessor = hygiene_outcome_card, public)]
#[derive(Clone, Debug)]
pub struct HygieneOutcomeCardRow {
    /// Action id shown to dashboard clients.
    #[primary_key]
    pub action_id: String,
    /// Compact actor label for display.
    pub recorded_by_label: String,
    /// Reviewed outcome label for display.
    pub outcome_label: String,
    /// Claimed or reviewed minutes saved.
    pub minutes_saved: u32,
    /// Whether protected live side effects remain blocked.
    pub live_delivery_allowed: bool,
    /// Source refs displayed for review traceability.
    pub source_record_refs: String,
    /// Issue refs displayed for review traceability.
    pub issue_refs: String,
}

impl HygieneOutcomeCardRow {
    /// Creates a public read-model row from already accepted storage facts.
    pub fn new(
        action_id: String,
        recorded_by_label: String,
        outcome_label: String,
        before_minutes: u32,
        actual_minutes: u32,
        source_record_refs: String,
        issue_refs: String,
    ) -> Self {
        Self {
            action_id,
            recorded_by_label,
            outcome_label,
            minutes_saved: before_minutes.saturating_sub(actual_minutes),
            live_delivery_allowed: false,
            source_record_refs,
            issue_refs,
        }
    }
}
