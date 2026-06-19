//! Daycare playgroup assignment decisions after eligibility and coverage are known.
//!
//! ```
//! use domain::{daycare, entities};
//! use uuid::Uuid;
//!
//! let request = daycare::assignment::Request::builder()
//!     .pet_id(entities::PetId(Uuid::nil()))
//!     .service(daycare::ServiceVariant::AllDayPlay)
//!     .eligibility(daycare::eligibility::GroupPlayDecision::Eligible {
//!         basis: daycare::eligibility::EligibleBasis::CurrentEvidence,
//!     })
//!     .coverage(daycare::coverage::Decision::Sufficient)
//!     .playgroup(daycare::assignment::PlaygroupId::try_new("small-dogs-am").unwrap())
//!     .build();
//!
//! assert!(matches!(
//!     daycare::assignment::Service.assign(request),
//!     daycare::assignment::Decision::Assigned { .. }
//! ));
//! ```

use super::*;
use crate::policy;

pub use playgroup_id::Id as PlaygroupId;

/// Playgroup identifier chosen from scheduling/source data for daycare assignment review.
pub mod playgroup_id {
    use super::*;

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
    pub struct Id(String);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Assignment request that joins pet, service, eligibility, coverage, and target playgroup evidence.
pub struct Request {
    /// Pet being considered for playgroup assignment.
    pub pet_id: PetId,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceVariant,
    /// Group-play eligibility decision that must be clear before assignment.
    pub eligibility: eligibility::GroupPlayDecision,
    /// Staffing coverage decision that must be sufficient before assignment.
    pub coverage: coverage::Decision,
    /// Candidate playgroup selected from resort operations data.
    pub playgroup: PlaygroupId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Playgroup assignment outcome staff can act on or review.
pub enum Decision {
    /// Pet may be placed in the requested playgroup.
    Assigned {
        /// Pet being considered for playgroup assignment.
        pet_id: PetId,
        /// Candidate playgroup selected from resort operations data.
        playgroup: PlaygroupId,
    },
    /// Pet is eligible, but staffing coverage prevents immediate assignment.
    Waitlist {
        /// Operational reason the assignment is not automatically clear.
        reason: WaitlistReason,
        /// Human review gate required before staff override this assignment outcome.
        gate: policy::ReviewGate,
    },
    /// Pet cannot be assigned until eligibility or safety review clears.
    Blocked {
        /// Operational reason the assignment is not automatically clear.
        reason: BlockReason,
        /// Human review gate required before staff override this assignment outcome.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons a daycare assignment should waitlist instead of placing the pet.
pub enum WaitlistReason {
    /// Staffing ratio or roster evidence does not support another playgroup assignment.
    StaffCoverageInsufficient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Reasons assignment is blocked rather than merely waitlisted.
pub enum BlockReason {
    /// Group-play eligibility is not clear enough for playgroup assignment.
    EligibilityNotCleared,
}

#[derive(Debug, Clone, Default)]
/// Deterministic daycare assignment service for playgroup placement decisions.
pub struct Service;

impl Service {
    /// Assigns, waitlists, or blocks a pet from playgroup placement using eligibility and coverage evidence.
    pub fn assign(&self, request: Request) -> Decision {
        match (&request.eligibility, &request.coverage) {
            (eligibility::GroupPlayDecision::Eligible { .. }, coverage::Decision::Sufficient) => {
                Decision::Assigned {
                    pet_id: request.pet_id,
                    playgroup: request.playgroup,
                }
            }
            (
                eligibility::GroupPlayDecision::Eligible { .. },
                coverage::Decision::Insufficient { gate, .. },
            ) => Decision::Waitlist {
                reason: WaitlistReason::StaffCoverageInsufficient,
                gate: gate.clone(),
            },
            (
                eligibility::GroupPlayDecision::Eligible { .. },
                coverage::Decision::Unknown { gate },
            ) => Decision::Waitlist {
                reason: WaitlistReason::StaffCoverageInsufficient,
                gate: gate.clone(),
            },
            _ => Decision::Blocked {
                reason: BlockReason::EligibilityNotCleared,
                gate: policy::ReviewGate::BehaviorReview,
            },
        }
    }
}
