//! SpacetimeDB table rows for private adapter storage.
//!
//! Rows in this module are persistence facts for the realtime adapter. They are
//! deliberately boring primitives and local enums; app/domain types are rebuilt
//! at the boundary before any semantic decision is made.

#![allow(
    missing_docs,
    reason = "SpacetimeDB table macros generate public accessors/traits without rustdoc hooks; row structs and fields remain documented."
)]

pub use crate::storage::review_queue::{
    BlockedActionAttemptRow, BlockedActionReasonColumn, DataQualityIssueRow, FeedbackOutcomeColumn,
    HygieneAuditEventRow, HygieneOutcomeRow, ResolutionStatusColumn, ReviewQueueItemRow,
    ReviewQueueStatusColumn, WorkflowEventRow, WorkflowOutcomeRow, blocked_action_attempt,
    data_quality_issue, hygiene_audit_event, hygiene_outcome, review_queue_item, workflow_event,
    workflow_outcome,
};

/// Staff/manager actor known to the realtime adapter.
#[spacetimedb::table(accessor = staff_actor)]
#[derive(Clone)]
pub struct StaffActorRow {
    /// App actor id submitted by clients or issued by an auth adapter.
    #[primary_key]
    pub actor_id: String,
    /// SpacetimeDB identity string that may resolve to this actor.
    #[index(btree)]
    pub identity: String,
    /// Domain actor kind used to rebuild `domain::entities::ActorRef`.
    pub actor_kind: ActorKindColumn,
    /// Domain actor id payload; interpreted according to `actor_kind`.
    pub actor_ref: String,
    /// Schema version for additive row evolution.
    pub schema_version: u32,
}

/// Review role assigned to an actor.
#[spacetimedb::table(accessor = role_assignment)]
#[derive(Clone)]
pub struct RoleAssignmentRow {
    /// Synthetic row id because SpacetimeDB does not support composite primary keys.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// App actor id.
    #[index(btree)]
    pub actor_id: String,
    /// Review role granted to this actor.
    pub review_role: ReviewerRoleColumn,
    /// Schema version for additive row evolution.
    pub schema_version: u32,
}

/// One legacy location scope for an actor.
///
/// This declaration preserves the deployed `location_scope` table shape exactly. Authorization
/// never reads it; only `location_scope_v1` can promote rows into runtime authority.
#[spacetimedb::table(accessor = location_scope)]
#[derive(Clone)]
pub struct LocationScopeRow {
    /// Synthetic row id because SpacetimeDB does not support composite primary keys.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// App actor id.
    #[index(btree)]
    pub actor_id: String,
    /// Location id this historical row covered.
    #[index(btree)]
    pub location_id: String,
}

/// Version-one location scope for an actor.
///
/// A new table identity avoids unsupported in-place column addition to the
/// legacy `location_scope` table during module upgrades.
#[spacetimedb::table(accessor = location_scope_v1)]
#[derive(Clone)]
pub struct LocationScopeV1Row {
    /// Synthetic row id because SpacetimeDB does not support composite primary keys.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// App actor id.
    #[index(btree)]
    pub actor_id: String,
    /// Location id this actor covers.
    #[index(btree)]
    pub location_id: String,
    /// Schema version for authorization-scope row evolution.
    pub schema_version: u32,
}

macro_rules! redacted_debug {
    ($($row:ty => $name:literal),+ $(,)?) => {
        $(
            impl std::fmt::Debug for $row {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(concat!($name, "([REDACTED])"))
                }
            }
        )+
    };
}

redacted_debug!(
    StaffActorRow => "StaffActorRow",
    RoleAssignmentRow => "RoleAssignmentRow",
    LocationScopeRow => "LocationScopeRow",
    LocationScopeV1Row => "LocationScopeV1Row",
);

/// Actor kind encoded in storage, not a domain entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum ActorKindColumn {
    /// Staff actor.
    Staff,
    /// Manager actor.
    Manager,
    /// System actor.
    System,
}

/// Review role encoded in storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, spacetimedb::SpacetimeType)]
pub enum ReviewerRoleColumn {
    /// General manager review role.
    GeneralManager,
    /// Assistant general manager review role.
    AssistantGeneralManager,
    /// Front desk lead review role.
    FrontDeskLead,
    /// Front desk agent review role.
    FrontDeskAgent,
    /// Regional operator review role.
    RegionalOperator,
    /// Operations analyst review role.
    OperationsAnalyst,
}
