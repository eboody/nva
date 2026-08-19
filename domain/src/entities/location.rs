//! Resort location identity, brand, capability, and policy references.

use serde::{Deserialize, Serialize};

use super::{identifiers::LocationId, reservation::ServiceKind};
use crate::{location, policy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Resort location record that scopes local capabilities, timezone, brand, and policy references.
pub struct Location {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: LocationId,
    /// Brand retained from source records for staff review, safety gates, and workflow joins.
    pub brand: Brand,
    /// Contact or display name used by staff.
    pub name: location::Name,
    /// Timezone retained from source records for staff review, safety gates, and workflow joins.
    pub timezone: location::Timezone,
    /// Capabilities retained from source records for staff review, safety gates, and workflow joins.
    pub capabilities: Vec<ServiceKind>,
    /// Policies retained from source records for staff review, safety gates, and workflow joins.
    pub policies: LocationPolicyRefs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Brand family used to group multi-site operating records without losing local resort identity.
pub enum Brand {
    /// Nva pet resorts state or source category preserved for normalized resort records.
    NvaPetResorts,
    /// Pet suites state or source category preserved for normalized resort records.
    PetSuites,
    /// Contact or display name used by staff.
    NeighborhoodPetResort {
        /// Name attached to this variant for reviewers and adapters.
        name: location::Name,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// References to the local policy set that controls automation, vaccine, and play-safety decisions.
pub struct LocationPolicyRefs {
    /// Vaccine policy id retained from source records for staff review, safety gates, and workflow joins.
    pub vaccine_policy_id: policy::Id,
    /// Deposit policy id retained from source records for staff review, safety gates, and workflow joins.
    pub deposit_policy_id: policy::Id,
    /// Playgroup policy id retained from source records for staff review, safety gates, and workflow joins.
    pub playgroup_policy_id: policy::Id,
}
