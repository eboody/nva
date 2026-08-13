//! Access, role, visibility, and allowed-use contracts for scoped operational facts.
//!
//! These values scope CRM/customer intelligence, assistant context, and review-gated operational
//! facts. They do not prove identity, consent, marketing eligibility, or live-action authority by
//! themselves; adapters and app workflows must still attach source evidence and review gates.

use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::staff;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        AsRef,
        Serialize,
        Deserialize
    )
)]
/// Stable actor id for a staff member, role account, or approved operator context.
pub struct ActorId(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize
    )
)]
/// Human title used when scoping assistant context and data access.
pub struct Title(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Role used to scope operational context and permitted assistant retrieval.
pub enum ActorRole {
    /// Front-desk staff handling lead, booking, checkout, and customer work.
    FrontDesk,
    /// Care/kennel team member handling pet care and facility tasks.
    CareStaff,
    /// Groomer role.
    Groomer,
    /// Trainer role.
    Trainer,
    /// Site manager or general manager.
    SiteManager,
    /// Regional operations role.
    RegionalOperations,
    /// Finance analyst or regional/site financial reviewer.
    Finance,
    /// Marketing role with restricted access to operations-only facts.
    Marketing,
}

impl TryFrom<ActorRole> for staff::Role {
    type Error = RolePromotionError;

    fn try_from(value: ActorRole) -> Result<Self, Self::Error> {
        match value {
            ActorRole::FrontDesk => Ok(Self::FrontDesk),
            ActorRole::CareStaff => Ok(Self::KennelTechnician),
            ActorRole::Groomer => Ok(Self::Groomer),
            ActorRole::Trainer => Ok(Self::Trainer),
            ActorRole::SiteManager => Ok(Self::Manager),
            ActorRole::RegionalOperations | ActorRole::Finance | ActorRole::Marketing => {
                Err(RolePromotionError::NotSiteLaborRole { role: value })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
/// Failures returned when strategic access roles are promoted into canonical site labor roles.
pub enum RolePromotionError {
    #[error("strategic actor role {role:?} is not a canonical site labor role")]
    /// Regional, finance, and marketing access roles do not own local staff-task labor.
    NotSiteLaborRole {
        /// Actor role that cannot be treated as site labor.
        role: ActorRole,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Visibility boundary for customer, pet, financial, and knowledge context.
pub enum VisibilityScope {
    /// Only operational staff with local/site context may use the fact.
    OperationsOnly,
    /// Marketing may use the fact after consent and segmentation review.
    MarketingEligible,
    /// Site managers may use the fact for local operations.
    SiteManagement,
    /// Regional users may use aggregated or approved cross-site facts.
    RegionalAggregate,
    /// Sensitive fact requires a specific review gate before use.
    ReviewGated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Purpose for which a fact may be used.
pub enum AllowedUse {
    /// Use to personalize service or staff handling.
    ServicePersonalization,
    /// Use for internal decision support only.
    InternalDecisionSupport,
    /// Use in a marketing campaign after consent and scope review.
    MarketingCampaign,
    /// Use for financial or performance reporting.
    FinancialAnalysis,
    /// Use for labor or staffing recommendation review.
    LaborOptimization,
}
