use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::{ServiceKind, Species};

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

pub mod automation {
    use serde::{Deserialize, Serialize};

    use super::WorkflowName;

    pub mod rationale {
        use nutype::nutype;

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
        pub struct Rationale(String);
    }

    pub use rationale::Rationale;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Level {
        SafeToAutomate,
        DraftOnly,
        InternalTaskOnly,
        ManagerApprovalRequired,
        NeverAutomate,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Rule {
        pub workflow: WorkflowName,
        pub level: Level,
        pub rationale: Rationale,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaccineRequirement {
    pub species: Species,
    pub service: ServiceKind,
    pub vaccines: Vec<VaccineName>,
    pub source_must_be_licensed_vet: bool,
}

pub mod play {
    pub use eligibility::{
        ConservativePolicy, Decision, Eligibility, IneligibilityReason, Policy, Reason,
    };

    pub mod eligibility {
        use serde::{Deserialize, Serialize};

        use crate::entities::{Pet, ServiceKind, SpayNeuterStatus, Species};

        use super::super::ReviewGate;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Decision {
            pub eligibility: Eligibility,
            pub required_review: Option<ReviewGate>,
        }

        impl Decision {
            pub fn eligible_for_group_play(&self) -> bool {
                matches!(self.eligibility, Eligibility::Eligible(_))
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Eligibility {
            Eligible(Reason),
            Ineligible(IneligibilityReason),
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum Reason {
            NoConservativeHardStop,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum IneligibilityReason {
            ServiceDoesNotRequireGroupPlay,
            SpeciesReceivesIndividualPlay,
            SpayNeuterStatusRequiresReview,
            BehaviorFlagsRequireReview,
        }

        impl std::fmt::Display for IneligibilityReason {
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

        pub trait Policy {
            fn decide(&self, pet: &Pet, service: &ServiceKind) -> Decision;
        }

        /// PetSuites/NVA-inspired default from public policy pages.
        ///
        /// This is intentionally conservative: it can route to day boarding / review, but it
        /// should not be the final source of truth for a live location's local policy.
        #[derive(Debug, Clone, Default)]
        pub struct ConservativePolicy;

        impl Policy for ConservativePolicy {
            fn decide(&self, pet: &Pet, service: &ServiceKind) -> Decision {
                if !matches!(service, ServiceKind::DayPlay | ServiceKind::Boarding) {
                    return Decision {
                        eligibility: Eligibility::Ineligible(
                            IneligibilityReason::ServiceDoesNotRequireGroupPlay,
                        ),
                        required_review: None,
                    };
                }

                if pet.species != Species::Dog {
                    return Decision {
                        eligibility: Eligibility::Ineligible(
                            IneligibilityReason::SpeciesReceivesIndividualPlay,
                        ),
                        required_review: None,
                    };
                }

                if matches!(
                    pet.spay_neuter_status,
                    SpayNeuterStatus::Intact | SpayNeuterStatus::Unknown
                ) {
                    return Decision {
                        eligibility: Eligibility::Ineligible(
                            IneligibilityReason::SpayNeuterStatusRequiresReview,
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
                    crate::temperament::Rating::ReviewRequired
                ) || pet.temperament.behavior_observations.iter().any(
                    crate::temperament::BehaviorObservation::indicates_behavior_review_evidence,
                ) {
                    return Decision {
                        eligibility: Eligibility::Ineligible(
                            IneligibilityReason::BehaviorFlagsRequireReview,
                        ),
                        required_review: Some(ReviewGate::BehaviorReview),
                    };
                }

                Decision {
                    eligibility: Eligibility::Eligible(Reason::NoConservativeHardStop),
                    required_review: None,
                }
            }
        }
    }
}

pub mod denial {
    use serde::{Deserialize, Serialize};

    use super::play;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Reason {
        ManagerApprovalRequired,
        MedicalDocumentReviewRequired,
        BehaviorReviewRequired,
        CustomerMessageApprovalRequired,
        RefundOrDepositException,
        PlayEligibility(play::IneligibilityReason),
    }

    impl std::fmt::Display for Reason {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewGate {
    ManagerApproval,
    MedicalDocumentReview,
    BehaviorReview,
    CustomerMessageApproval,
    RefundOrDepositException,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{self, CustomerId, Pet, PetId, SpayNeuterStatus, TemperamentProfile};
    use crate::policy::play::Policy;
    use crate::temperament::{BehaviorObservation, GroupPlayObservation, Rating};
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
            care_profile: entities::CareProfile::default(),
        }
    }

    #[test]
    fn intact_dog_routes_away_from_group_play() {
        let decision =
            play::ConservativePolicy.decide(&dog(SpayNeuterStatus::Intact), &ServiceKind::DayPlay);
        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Ineligible(
                play::IneligibilityReason::SpayNeuterStatusRequiresReview
            )
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn neutered_dog_can_be_group_play_candidate() {
        let decision = play::ConservativePolicy
            .decide(&dog(SpayNeuterStatus::Neutered), &ServiceKind::DayPlay);
        assert!(decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Eligible(play::Reason::NoConservativeHardStop)
        );
    }

    #[test]
    fn bite_history_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament
            .behavior_observations
            .push(BehaviorObservation::BiteHistory);

        let decision = play::ConservativePolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Ineligible(play::IneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn explicit_manager_review_flag_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament
            .behavior_observations
            .push(BehaviorObservation::RequiresManagerReview);

        let decision = play::ConservativePolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Ineligible(play::IneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn staff_evaluation_observation_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.group_play_observation = GroupPlayObservation::NeedsIntroAssessment;

        let decision = play::ConservativePolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Ineligible(play::IneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn observed_group_stress_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.group_play_observation = GroupPlayObservation::StressedInGroupSetting;

        let decision = play::ConservativePolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Ineligible(play::IneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }

    #[test]
    fn comfortable_group_observation_has_no_conservative_hard_stop() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.group_play_observation = GroupPlayObservation::ComfortableInObservedGroup;

        let decision = play::ConservativePolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Eligible(play::Reason::NoConservativeHardStop)
        );
    }

    #[test]
    fn play_eligibility_denial_reason_displays_group_play_context() {
        let reason =
            denial::Reason::PlayEligibility(play::IneligibilityReason::BehaviorFlagsRequireReview);

        assert_eq!(
            reason.to_string(),
            "play eligibility denied: behavior flags require review"
        );
    }

    #[test]
    fn review_required_temperament_rating_requires_behavior_review() {
        let mut pet = dog(SpayNeuterStatus::Neutered);
        pet.temperament.rating = Rating::ReviewRequired;

        let decision = play::ConservativePolicy.decide(&pet, &ServiceKind::DayPlay);

        assert!(!decision.eligible_for_group_play());
        assert_eq!(
            decision.eligibility,
            play::Eligibility::Ineligible(play::IneligibilityReason::BehaviorFlagsRequireReview)
        );
        assert_eq!(decision.required_review, Some(ReviewGate::BehaviorReview));
    }
}
