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

/// Playgroup id boundary for daycare assignment contracts.
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
/// Typed request domain value that keeps raw primitives out of daycare workflows.
pub struct Request {
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceVariant,
    /// Eligibility fact promoted into this daycare contract.
    pub eligibility: eligibility::GroupPlayDecision,
    /// Coverage fact promoted into this daycare contract.
    pub coverage: coverage::Decision,
    /// Playgroup fact promoted into this daycare contract.
    pub playgroup: PlaygroupId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for decision decisions in daycare workflows.
pub enum Decision {
    /// Assigned daycare attendance, eligibility, coverage, or package signal.
    Assigned {
        /// Pet receiving the grooming or care service.
        pet_id: PetId,
        /// Playgroup fact promoted into this daycare contract.
        playgroup: PlaygroupId,
    },
    /// Waitlist daycare attendance, eligibility, coverage, or package signal.
    Waitlist {
        /// Business reason staff should review before proceeding.
        reason: WaitlistReason,
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
    /// Blocked daycare attendance, eligibility, coverage, or package signal.
    Blocked {
        /// Business reason staff should review before proceeding.
        reason: BlockReason,
        /// Gate fact promoted into this daycare contract.
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for waitlist reason decisions in daycare workflows.
pub enum WaitlistReason {
    /// Staff coverage insufficient daycare attendance, eligibility, coverage, or package signal.
    StaffCoverageInsufficient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for block reason decisions in daycare workflows.
pub enum BlockReason {
    /// Eligibility not cleared daycare attendance, eligibility, coverage, or package signal.
    EligibilityNotCleared,
}

#[derive(Debug, Clone, Default)]
/// Typed service domain value that keeps raw primitives out of daycare workflows.
pub struct Service;

impl Service {
    /// Returns the assign for this daycare value.
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
