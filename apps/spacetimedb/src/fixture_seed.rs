//! Target-independent plans for internal-only demo fixture reducers.
//!
//! The plans own authorization and deterministic row intent. Runtime adapters
//! supply storage mechanics, so native tests and the deployed SpacetimeDB path
//! execute the same transition without substituting target-specific semantics.

use crate::{
    storage::review_queue::codec,
    tables::{
        ActorKindColumn, LocationScopeRow, ReviewerRoleColumn, RoleAssignmentRow, StaffActorRow,
    },
};

/// Whether the reducer call originated inside the database runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authorization {
    /// Trusted database-internal fixture invocation.
    Internal,
    /// External client invocation, which must fail closed.
    External,
}

impl From<bool> for Authorization {
    fn from(is_internal: bool) -> Self {
        if is_internal {
            Self::Internal
        } else {
            Self::External
        }
    }
}

/// Complete semantic input for one demo actor seed transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSeed {
    /// Stable app actor identifier.
    pub actor_id: String,
    /// SpacetimeDB caller identity rendered in its canonical textual form.
    pub identity: String,
    /// Actor category used by authorization promotion.
    pub actor_kind: ActorKindColumn,
    /// Typed actor reference payload.
    pub actor_ref: String,
    /// Review role assigned by this fixture event.
    pub review_role: ReviewerRoleColumn,
    /// Location scope assigned by this fixture event.
    pub location_id: String,
}

/// Storage operations required by the actor seed plan.
pub trait ActorStore {
    /// Inserts or replaces the canonical actor row by actor id.
    fn upsert_actor(&mut self, row: StaffActorRow);
    /// Appends a role-assignment event; the store owns generated identifiers.
    fn insert_role(&mut self, row: RoleAssignmentRow);
    /// Appends a location-scope event; the store owns generated identifiers.
    fn insert_scope(&mut self, row: LocationScopeRow);
}

/// Fail-closed fixture seed rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// External callers cannot seed privileged role/location fixtures.
    ExternalCaller,
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ExternalCaller => {
                formatter.write_str("demo fixture seeding is restricted to internal database calls")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Executes the canonical actor seed transition against a storage port.
pub fn execute_actor_seed(
    store: &mut impl ActorStore,
    authorization: Authorization,
    seed: ActorSeed,
) -> Result<(), Error> {
    if authorization != Authorization::Internal {
        return Err(Error::ExternalCaller);
    }

    store.upsert_actor(StaffActorRow {
        actor_id: seed.actor_id.clone(),
        identity: seed.identity,
        actor_kind: seed.actor_kind,
        actor_ref: seed.actor_ref,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    store.insert_role(RoleAssignmentRow {
        id: 0,
        actor_id: seed.actor_id.clone(),
        review_role: seed.review_role,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    store.insert_scope(LocationScopeRow {
        id: 0,
        actor_id: seed.actor_id,
        location_id: seed.location_id,
        schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
    });
    Ok(())
}
