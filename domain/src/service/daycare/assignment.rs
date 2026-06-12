use super::*;
use crate::policy;

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
pub struct PlaygroupId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Request {
    pub pet_id: PetId,
    pub service: ServiceVariant,
    pub eligibility: eligibility::GroupPlayDecision,
    pub coverage: coverage::Decision,
    pub playgroup: PlaygroupId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Assigned {
        pet_id: PetId,
        playgroup: PlaygroupId,
    },
    Waitlist {
        reason: WaitlistReason,
        gate: policy::ReviewGate,
    },
    Blocked {
        reason: BlockReason,
        gate: policy::ReviewGate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaitlistReason {
    StaffCoverageInsufficient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockReason {
    EligibilityNotCleared,
}

#[derive(Debug, Clone, Default)]
pub struct Service;

impl Service {
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
