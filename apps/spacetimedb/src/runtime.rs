//! Runtime assembly for SpacetimeDB reducers.
//!
//! Reducers use this module to assemble app-owned ports from table snapshots and
//! then persist only the rows emitted by those ports.

use app::data_quality_hygiene as hygiene;
use spacetimedb::{ReducerContext, Table};

use crate::{
    adapter::{
        ActorDirectoryAdapter, AuditLogAdapter, BlockedActionLogAdapter, OutcomeRecorderAdapter,
        ReviewQueueAdapter,
    },
    authz,
    read_model::staff_queue_item::blocked_action_notice,
    storage::review_queue::codec,
    tables::{
        LocationScopeRow, ReviewQueueItemRow, RoleAssignmentRow, StaffActorRow,
        blocked_action_attempt, hygiene_audit_event, hygiene_outcome, location_scope,
        review_queue_item, role_assignment, staff_actor,
    },
};

/// App service and writable adapter ports assembled for one reducer call.
pub struct HygieneCaptureRuntime {
    service: hygiene::OutcomeCaptureService<
        ActorDirectoryAdapter,
        hygiene::RoleLocationAuthorization,
        ReviewQueueAdapter,
        OutcomeRecorderAdapter,
        AuditLogAdapter,
        BlockedActionLogAdapter,
    >,
}

impl HygieneCaptureRuntime {
    /// Loads SpacetimeDB rows into app-owned ports for one reducer transaction.
    pub fn load(ctx: &ReducerContext) -> Self {
        let actor_directory = ActorDirectoryAdapter::new(
            ctx.db.staff_actor().iter().collect::<Vec<StaffActorRow>>(),
            ctx.db
                .role_assignment()
                .iter()
                .collect::<Vec<RoleAssignmentRow>>(),
            ctx.db
                .location_scope()
                .iter()
                .collect::<Vec<LocationScopeRow>>(),
        );
        let review_items = ctx
            .db
            .review_queue_item()
            .iter()
            .collect::<Vec<ReviewQueueItemRow>>();
        let review_queue = ReviewQueueAdapter::new(review_items.clone());
        Self {
            service: hygiene::OutcomeCaptureService::new(
                actor_directory,
                hygiene::RoleLocationAuthorization,
                review_queue,
                OutcomeRecorderAdapter::default(),
                AuditLogAdapter::default(),
                BlockedActionLogAdapter::new(review_items),
            ),
        }
    }

    /// Resolves the reducer caller identity into an app actor id.
    pub fn actor_id_for_sender(ctx: &ReducerContext) -> Option<hygiene::ActorId> {
        let identity = ctx.sender().to_string();
        let actor_rows = ctx.db.staff_actor().iter().collect::<Vec<StaffActorRow>>();
        authz::actor_id_for_identity(&identity, actor_rows.iter())
    }

    /// Invokes the app service and writes adapter rows back into SpacetimeDB.
    pub fn record_reviewed_outcome(
        mut self,
        ctx: &ReducerContext,
        request: hygiene::OutcomeCaptureRequest,
    ) -> hygiene::Result<hygiene::OutcomeReceipt> {
        let result = self.service.record_reviewed_outcome(request);
        let outcome_rows = self.service.outcome_recorder().rows();
        let audit_rows = self.service.audit_log().rows();
        let blocked_rows = self.service.blocked_action_log().rows();

        for row in outcome_rows {
            ctx.db.hygiene_outcome().insert(row);
        }
        for row in audit_rows {
            ctx.db.hygiene_audit_event().insert(row);
        }
        for row in blocked_rows {
            ctx.db.blocked_action_attempt().insert(row.clone());
            ctx.db
                .blocked_action_notice()
                .try_insert(codec::blocked_action_notice(&row))
                .ok();
        }

        result
    }
}
