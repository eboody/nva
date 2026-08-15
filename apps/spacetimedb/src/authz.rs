//! Identity-to-actor helpers for SpacetimeDB reducer boundaries.
//!
//! This module contains only adapter lookup/promotion logic. Authorization policy
//! remains app-owned (`app::data_quality_hygiene::RoleLocationAuthorization`).

use app::data_quality_hygiene as hygiene;
use domain::entities;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::{
    storage::review_queue::codec,
    tables::{
        ActorKindColumn, LocationScopeRow, LocationScopeV1Row, ReviewerRoleColumn,
        RoleAssignmentRow, StaffActorRow,
    },
};

/// Persisted-authority defects that prevent safe application actor rehydration.
///
/// Every variant fails closed. In particular, row iteration order never chooses
/// between duplicate identities or role assignments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RehydrationError {
    /// More than one actor row claimed the same authenticated identity.
    DuplicateIdentity,
    /// The persisted actor id or actor reference violated its semantic constructor.
    MalformedActor,
    /// No role assignment exists for the actor.
    MissingRole,
    /// Multiple role rows disagree, and no multi-role policy is currently owned.
    ConflictingRoles,
    /// Duplicate role rows exist even though authority cardinality is exactly one.
    DuplicateRoleAssignment,
    /// At least one persisted location scope could not be promoted.
    MalformedLocationScope,
    /// An authority-bearing row uses a schema version this adapter does not understand.
    UnsupportedSchemaVersion,
    /// The requested actor did not match the authenticated database identity.
    AuthenticatedActorMismatch,
}

/// Resolves a SpacetimeDB identity string into an app actor id.
///
/// Absence is distinct from malformed or ambiguous persisted authority.
pub fn actor_id_for_identity<'a>(
    identity: &str,
    rows: impl IntoIterator<Item = &'a StaffActorRow>,
) -> Result<Option<hygiene::ActorId>, RehydrationError> {
    let mut matches = rows.into_iter().filter(|row| row.identity == identity);
    let Some(row) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(RehydrationError::DuplicateIdentity);
    }
    validate_schema_version(row.schema_version)?;
    hygiene::ActorId::try_new(row.actor_id.clone())
        .map(Some)
        .map_err(|_| RehydrationError::MalformedActor)
}

/// Plans an additive v1 backfill for only the exact authenticated actor.
///
/// Actor, role, current-v1 authority, and every matching legacy location are validated before any
/// row is returned, allowing the reducer transaction to insert the complete plan or fail closed.
pub fn authenticated_legacy_scope_migration(
    authenticated_identity: &str,
    expected_actor_id: &hygiene::ActorId,
    actor_rows: &[StaffActorRow],
    role_rows: &[RoleAssignmentRow],
    legacy_rows: &[LocationScopeRow],
    v1_rows: &[LocationScopeV1Row],
) -> Result<Vec<LocationScopeV1Row>, RehydrationError> {
    let authenticated_actor_id = actor_id_for_identity(authenticated_identity, actor_rows.iter())?
        .ok_or(RehydrationError::AuthenticatedActorMismatch)?;
    if &authenticated_actor_id != expected_actor_id {
        return Err(RehydrationError::AuthenticatedActorMismatch);
    }
    let actor_row = actor_rows
        .iter()
        .find(|row| row.actor_id == expected_actor_id.as_ref())
        .ok_or(RehydrationError::MalformedActor)?;
    actor_assignment_from_rows(actor_row, role_rows.iter(), v1_rows.iter())?;

    let mut seen_locations = BTreeSet::new();
    let mut pending = Vec::new();
    for row in legacy_rows
        .iter()
        .filter(|row| row.actor_id == expected_actor_id.as_ref())
    {
        parse_location_id(&row.location_id).ok_or(RehydrationError::MalformedLocationScope)?;
        if !seen_locations.insert(row.location_id.clone())
            || v1_rows.iter().any(|current| {
                current.actor_id == row.actor_id && current.location_id == row.location_id
            })
        {
            continue;
        }
        pending.push(LocationScopeV1Row {
            id: 0,
            actor_id: row.actor_id.clone(),
            location_id: row.location_id.clone(),
            schema_version: codec::REVIEW_QUEUE_SCHEMA_VERSION,
        });
    }
    Ok(pending)
}

/// Promotes exactly one compatible actor, role, and complete scope set.
///
/// The adapter intentionally rejects multiple roles until application policy
/// explicitly owns multi-role semantics.
pub fn actor_assignment_from_rows<'a>(
    actor_row: &StaffActorRow,
    role_rows: impl IntoIterator<Item = &'a RoleAssignmentRow>,
    scope_rows: impl IntoIterator<Item = &'a LocationScopeV1Row>,
) -> Result<hygiene::ActorAssignment, RehydrationError> {
    validate_schema_version(actor_row.schema_version)?;
    let actor_id = hygiene::ActorId::try_new(actor_row.actor_id.clone())
        .map_err(|_| RehydrationError::MalformedActor)?;
    let actor = actor_ref_from_row(actor_row).ok_or(RehydrationError::MalformedActor)?;

    let matching_roles = role_rows
        .into_iter()
        .filter(|role| role.actor_id == actor_row.actor_id)
        .collect::<Vec<_>>();
    let [role] = matching_roles.as_slice() else {
        return Err(match matching_roles.as_slice() {
            [] => RehydrationError::MissingRole,
            roles
                if roles
                    .windows(2)
                    .any(|pair| pair[0].review_role != pair[1].review_role) =>
            {
                RehydrationError::ConflictingRoles
            }
            _ => RehydrationError::DuplicateRoleAssignment,
        });
    };
    validate_schema_version(role.schema_version)?;
    let review_role = review_role_from_column(role.review_role);

    let location_ids = scope_rows
        .into_iter()
        .filter(|scope| scope.actor_id == actor_row.actor_id)
        .map(|scope| {
            validate_schema_version(scope.schema_version)?;
            parse_location_id(&scope.location_id).ok_or(RehydrationError::MalformedLocationScope)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(hygiene::ActorAssignment::new(
        actor_id,
        actor,
        review_role,
        location_ids,
    ))
}

fn validate_schema_version(schema_version: u32) -> Result<(), RehydrationError> {
    (schema_version == codec::REVIEW_QUEUE_SCHEMA_VERSION)
        .then_some(())
        .ok_or(RehydrationError::UnsupportedSchemaVersion)
}

fn actor_ref_from_row(row: &StaffActorRow) -> Option<entities::ActorRef> {
    match row.actor_kind {
        ActorKindColumn::Staff => Some(entities::ActorRef::Staff {
            staff_id: entities::StaffId::try_new(row.actor_ref.clone()).ok()?,
        }),
        ActorKindColumn::Manager => Some(entities::ActorRef::Manager {
            manager_id: entities::ManagerId::try_new(row.actor_ref.clone()).ok()?,
        }),
        ActorKindColumn::System => Some(entities::ActorRef::System),
    }
}

/// Maps a row role column into the app review role enum.
pub const fn review_role_from_column(role: ReviewerRoleColumn) -> hygiene::ReviewerRole {
    match role {
        ReviewerRoleColumn::GeneralManager => hygiene::ReviewerRole::GeneralManager,
        ReviewerRoleColumn::AssistantGeneralManager => {
            hygiene::ReviewerRole::AssistantGeneralManager
        }
        ReviewerRoleColumn::FrontDeskLead => hygiene::ReviewerRole::FrontDeskLead,
        ReviewerRoleColumn::FrontDeskAgent => hygiene::ReviewerRole::FrontDeskAgent,
        ReviewerRoleColumn::RegionalOperator => hygiene::ReviewerRole::RegionalOperator,
        ReviewerRoleColumn::OperationsAnalyst => hygiene::ReviewerRole::OperationsAnalyst,
    }
}

/// Parses a location id into the domain location id.
///
/// Production paths use UUIDs. Demo fixtures may use compact location numbers like
/// `101`; those are promoted into deterministic UUID values by placing the number
/// in the low bits.
pub fn parse_location_id(raw: &str) -> Option<entities::LocationId> {
    if let Ok(uuid) = Uuid::parse_str(raw) {
        return entities::LocationId::try_new(uuid).ok();
    }
    raw.parse::<u128>()
        .ok()
        .and_then(|value| entities::LocationId::try_new(Uuid::from_u128(value)).ok())
}
