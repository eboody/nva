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

/// Automation boundary for policy contracts.
pub mod automation {
    use serde::{Deserialize, Serialize};

    use super::WorkflowName;

    /// Rationale boundary for policy contracts.
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
    /// Domain vocabulary for level decisions in policy workflows.
    pub enum Level {
        /// Safe to automate policy decision or approval gate.
        SafeToAutomate,
        /// Draft only policy decision or approval gate.
        DraftOnly,
        /// Internal task only policy decision or approval gate.
        InternalTaskOnly,
        /// Manager approval required policy decision or approval gate.
        ManagerApprovalRequired,
        /// Never automate policy decision or approval gate.
        NeverAutomate,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed rule domain value that keeps raw primitives out of policy workflows.
    pub struct Rule {
        /// Workflow fact promoted into this policy contract.
        pub workflow: WorkflowName,
        /// Level fact promoted into this policy contract.
        pub level: Level,
        /// Rationale fact promoted into this policy contract.
        pub rationale: Rationale,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed vaccine requirement domain value that keeps raw primitives out of policy workflows.
pub struct VaccineRequirement {
    /// Species fact promoted into this policy contract.
    pub species: Species,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceKind,
    /// Vaccines fact promoted into this policy contract.
    pub vaccines: Vec<VaccineName>,
    /// Source must be licensed vet fact promoted into this policy contract.
    pub source_must_be_licensed_vet: bool,
}

/// Play boundary for policy contracts.
pub mod play {
    pub use eligibility::{
        ConservativePolicy, Decision, Eligibility, IneligibilityReason, Policy, Reason,
    };

    /// Eligibility boundary for policy contracts.
    pub mod eligibility {
        use serde::{Deserialize, Serialize};

        use crate::entities::{Pet, ServiceKind, SpayNeuterStatus, Species};

        use super::super::ReviewGate;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Typed decision domain value that keeps raw primitives out of policy workflows.
        pub struct Decision {
            /// Eligibility fact promoted into this policy contract.
            pub eligibility: Eligibility,
            /// Required review fact promoted into this policy contract.
            pub required_review: Option<ReviewGate>,
        }

        impl Decision {
            /// Returns the eligible for group play for this policy value.
            pub fn eligible_for_group_play(&self) -> bool {
                matches!(self.eligibility, Eligibility::Eligible(_))
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Domain vocabulary for eligibility decisions in policy workflows.
        pub enum Eligibility {
            /// Eligible policy decision or approval gate.
            Eligible(Reason),
            /// Ineligible policy decision or approval gate.
            Ineligible(IneligibilityReason),
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Domain vocabulary for reason decisions in policy workflows.
        pub enum Reason {
            /// No conservative hard stop policy decision or approval gate.
            NoConservativeHardStop,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Domain vocabulary for ineligibility reason decisions in policy workflows.
        pub enum IneligibilityReason {
            /// Service does not require group play policy decision or approval gate.
            ServiceDoesNotRequireGroupPlay,
            /// Species receives individual play policy decision or approval gate.
            SpeciesReceivesIndividualPlay,
            /// Spay neuter status requires review policy decision or approval gate.
            SpayNeuterStatusRequiresReview,
            /// Behavior flags require review policy decision or approval gate.
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

        /// Defines the behavior required from a policy participant in the policy workflow.
        pub trait Policy {
            /// Returns the pet for this policy value.
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

/// Denial boundary for policy contracts.
pub mod denial {
    use serde::{Deserialize, Serialize};

    use super::play;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for reason decisions in policy workflows.
    pub enum Reason {
        /// Manager approval required policy decision or approval gate.
        ManagerApprovalRequired,
        /// Medical document review required policy decision or approval gate.
        MedicalDocumentReviewRequired,
        /// Behavior history requires review before service.
        BehaviorReviewRequired,
        /// Customer message approval required policy decision or approval gate.
        CustomerMessageApprovalRequired,
        /// Refund or deposit exception policy decision or approval gate.
        RefundOrDepositException,
        /// Play eligibility policy decision or approval gate.
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
/// Domain vocabulary for review gate decisions in policy workflows.
pub enum ReviewGate {
    /// Manager approval policy decision or approval gate.
    ManagerApproval,
    /// Medical document review policy decision or approval gate.
    MedicalDocumentReview,
    /// Behavior review policy decision or approval gate.
    BehaviorReview,
    /// Customer message approval policy decision or approval gate.
    CustomerMessageApproval,
    /// Refund or deposit exception policy decision or approval gate.
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
