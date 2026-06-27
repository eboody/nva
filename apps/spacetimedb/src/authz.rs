//! Identity-to-actor helpers for SpacetimeDB reducer boundaries.
//!
//! This module contains only adapter lookup/promotion logic. Authorization policy
//! remains app-owned (`app::data_quality_hygiene::RoleLocationAuthorization`).

use app::data_quality_hygiene as hygiene;
use domain::entities;
use uuid::Uuid;

use crate::tables::{
    ActorKindColumn, LocationScopeRow, ReviewerRoleColumn, RoleAssignmentRow, StaffActorRow,
};

/// Resolves a SpacetimeDB identity string into an app actor id.
pub fn actor_id_for_identity<'a>(
    identity: &str,
    rows: impl IntoIterator<Item = &'a StaffActorRow>,
) -> Option<hygiene::ActorId> {
    rows.into_iter()
        .find(|row| row.identity == identity)
        .and_then(|row| hygiene::ActorId::try_new(row.actor_id.clone()).ok())
}

/// Promotes storage rows into an app actor assignment.
pub fn actor_assignment_from_rows<'a>(
    actor_row: &StaffActorRow,
    role_rows: impl IntoIterator<Item = &'a RoleAssignmentRow>,
    scope_rows: impl IntoIterator<Item = &'a LocationScopeRow>,
) -> Option<hygiene::ActorAssignment> {
    let actor_id = hygiene::ActorId::try_new(actor_row.actor_id.clone()).ok()?;
    let actor = actor_ref_from_row(actor_row)?;
    let review_role = review_role_from_column(
        role_rows
            .into_iter()
            .find(|role| role.actor_id == actor_row.actor_id)?
            .review_role,
    );
    let location_ids = scope_rows
        .into_iter()
        .filter(|scope| scope.actor_id == actor_row.actor_id)
        .filter_map(|scope| parse_location_id(&scope.location_id))
        .collect();

    Some(hygiene::ActorAssignment::new(
        actor_id,
        actor,
        review_role,
        location_ids,
    ))
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
        return Some(entities::LocationId(uuid));
    }
    raw.parse::<u128>()
        .ok()
        .map(Uuid::from_u128)
        .map(entities::LocationId)
}
