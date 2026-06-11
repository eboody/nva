use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::{Pet, ServiceKind, SpayNeuterStatus, Species};

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

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 80),
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
pub struct VaccineName(String);

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
pub struct WorkflowName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
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
pub struct AutomationRationale(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaccineRequirement {
    pub species: Species,
    pub service: ServiceKind,
    pub vaccines: Vec<VaccineName>,
    pub source_must_be_licensed_vet: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayEligibilityDecision {
    pub eligibility: PlayEligibility,
    pub required_review: Option<ReviewGate>,
}

impl PlayEligibilityDecision {
    pub fn eligible_for_group_play(&self) -> bool {
        matches!(self.eligibility, PlayEligibility::Eligible(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayEligibility {
    Eligible(PlayEligibilityReason),
    Ineligible(PlayIneligibilityReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayEligibilityReason {
    NoConservativeHardStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayIneligibilityReason {
    ServiceDoesNotRequireGroupPlay,
    SpeciesReceivesIndividualPlay,
    SpayNeuterStatusRequiresReview,
    BehaviorFlagsRequireReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyDenialReason {
    ManagerApprovalRequired,
    MedicalDocumentReviewRequired,
    BehaviorReviewRequired,
    CustomerMessageApprovalRequired,
    RefundOrDepositException,
    PlayEligibility(PlayIneligibilityReason),
}

impl std::fmt::Display for PolicyDenialReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::ManagerApprovalRequired => "manager approval required",
            Self::MedicalDocumentReviewRequired => "medical document review required",
            Self::BehaviorReviewRequired => "behavior review required",
            Self::CustomerMessageApprovalRequired => "customer message approval required",
            Self::RefundOrDepositException => "refund or deposit exception",
            Self::PlayEligibility(reason) => {
                return write!(formatter, "play eligibility denied: {reason}");
            }
        };
        formatter.write_str(label)
    }
}

impl std::fmt::Display for PlayIneligibilityReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::ServiceDoesNotRequireGroupPlay => "service does not require group play",
            Self::SpeciesReceivesIndividualPlay => "species receives individual play",
            Self::SpayNeuterStatusRequiresReview => "spay/neuter status requires review",
            Self::BehaviorFlagsRequireReview => "behavior flags require review",
        };
        formatter.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewGate {
    ManagerApproval,
    MedicalDocumentReview,
    BehaviorReview,
    CustomerMessageApproval,
    RefundOrDepositException,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationLevel {
    SafeToAutomate,
    DraftOnly,
    InternalTaskOnly,
    ManagerApprovalRequired,
    NeverAutomate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationRule {
    pub workflow: WorkflowName,
    pub level: AutomationLevel,
    pub rationale: AutomationRationale,
}

pub trait PlayEligibilityPolicy {
    fn decide(&self, pet: &Pet, service: &ServiceKind) -> PlayEligibilityDecision;
}

/// PetSuites/NVA-inspired default from public policy pages.
///
/// This is intentionally conservative: it can route to day boarding / review, but it
/// should not be the final source of truth for a live location's local policy.
#[derive(Debug, Clone, Default)]
pub struct ConservativePlayEligibilityPolicy;

impl PlayEligibilityPolicy for ConservativePlayEligibilityPolicy {
    fn decide(&self, pet: &Pet, service: &ServiceKind) -> PlayEligibilityDecision {
        if !matches!(service, ServiceKind::DayPlay | ServiceKind::Boarding) {
            return PlayEligibilityDecision {
                eligibility: PlayEligibility::Ineligible(
                    PlayIneligibilityReason::ServiceDoesNotRequireGroupPlay,
                ),
                required_review: None,
            };
        }

        if pet.species != Species::Dog {
            return PlayEligibilityDecision {
                eligibility: PlayEligibility::Ineligible(
                    PlayIneligibilityReason::SpeciesReceivesIndividualPlay,
                ),
                required_review: None,
            };
        }

        if matches!(
            pet.spay_neuter_status,
            SpayNeuterStatus::Intact | SpayNeuterStatus::Unknown
        ) {
            return PlayEligibilityDecision {
                eligibility: PlayEligibility::Ineligible(
                    PlayIneligibilityReason::SpayNeuterStatusRequiresReview,
                ),
                required_review: Some(ReviewGate::BehaviorReview),
            };
        }

        if matches!(
            pet.temperament.group_play_observation,
            crate::temperament::GroupPlayObservation::StressedInGroupSetting
                | crate::temperament::GroupPlayObservation::NeedsIntroAssessment
        ) || matches!(
            pet.temperament.rating,
            crate::temperament::TemperamentRating::ReviewRequired
        ) || pet
            .temperament
            .behavior_observations
            .iter()
            .any(crate::temperament::BehaviorObservation::indicates_behavior_review_evidence)
        {
            return PlayEligibilityDecision {
                eligibility: PlayEligibility::Ineligible(
                    PlayIneligibilityReason::BehaviorFlagsRequireReview,
                ),
                required_review: Some(ReviewGate::BehaviorReview),
            };
        }

        PlayEligibilityDecision {
            eligibility: PlayEligibility::Eligible(PlayEligibilityReason::NoConservativeHardStop),
            required_review: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{CareProfile, CustomerId, PetId, TemperamentProfile};
    use crate::temperament::{BehaviorObservation, GroupPlayObservation, TemperamentRating};
    use uuid::Uuid;

    fn dog(spay_neuter_status: SpayNeuterStatus) -> Pet {
        Pet {
            id: PetId(Uuid::new_v4()),
            customer_id: CustomerId(Uuid::new_v4()),
            name: crate::pet::Name::try_new("Moose").expect("test pet name is valid"),
            species: Species::Dog,
            birth_date: None,
            sex: None,
            spay_neuter_status,
            temperament: TemperamentProfile::default(),
            care_profile: CareProfile::default(),
        }
    }

    #[test]
    fn intact_dog_routes_away_from_group_play() {
        let decision = ConservativePlayEligibilityPolicy
            .decide(&dog(SpayNeuterStatus::Intact), &ServiceKind::DayPlay);
        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Ineligible(PlayIneligibilityReason::SpayNeuterStatusRequiresReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn neutered_dog_can_be_group_play_candidate() {
        let decision = ConservativePlayEligibilityPolicy
            .decide(&dog(SpayNeuterStatus::Neutered), &ServiceKind::DayPlay);
        assert!(decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Eligible(PlayEligibilityReason::NoConservativeHardStop)
        );
    }

    #[test]
    fn bite_history_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament
            .behavior_observations
            .push(BehaviorObservation::BiteHistory);

        let decision = ConservativePlayEligibilityPolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Ineligible(PlayIneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn explicit_manager_review_flag_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament
            .behavior_observations
            .push(BehaviorObservation::RequiresManagerReview);

        let decision = ConservativePlayEligibilityPolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Ineligible(PlayIneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn staff_evaluation_observation_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.group_play_observation = GroupPlayObservation::NeedsIntroAssessment;

        let decision = ConservativePlayEligibilityPolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Ineligible(PlayIneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn observed_group_stress_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.group_play_observation = GroupPlayObservation::StressedInGroupSetting;

        let decision = ConservativePlayEligibilityPolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Ineligible(PlayIneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn comfortable_group_observation_has_no_conservative_hard_stop() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.group_play_observation = GroupPlayObservation::ComfortableInObservedGroup;

        let decision = ConservativePlayEligibilityPolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Eligible(PlayEligibilityReason::NoConservativeHardStop)
        );
    }

    #[test]
    fn review_required_temperament_rating_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.rating = TemperamentRating::ReviewRequired;

        let decision = ConservativePlayEligibilityPolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            PlayEligibility::Ineligible(PlayIneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }
}
