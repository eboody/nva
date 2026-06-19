//! Policy gates that decide what automation may do safely.
//!
//! Policy values encode the operating line between labor-saving automation and human review:
//! group-play eligibility, vaccine requirements, manager approval, medical-document review,
//! customer-message approval, and refund/deposit exceptions. These types document why an agent may
//! draft, route, suppress, or escalate work; they do not grant permission to override local resort
//! policy or invent availability.

use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::entities::{ServiceKind, Species};

/// Stable policy identifier used to reference local resort, brand, or portfolio rule sets.
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

/// Vaccine name as it appears in a requirement, proof document, or source-system mapping.
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

/// Named workflow that an automation-safety rule governs.
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

/// Automation-level policy rules that classify workflows as safe, draft-only, internal, or review-gated.
pub mod automation {
    use serde::{Deserialize, Serialize};

    use super::WorkflowName;

    /// Rationale text explaining the operational reason for an automation policy decision.
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
    /// Automation authority level that says whether a workflow may run, draft, create internal tasks, or require approval.
    pub enum Level {
        /// Safe to automate outcome in the automation authority or human-review policy.
        SafeToAutomate,
        /// Draft only outcome in the automation authority or human-review policy.
        DraftOnly,
        /// Internal task only outcome in the automation authority or human-review policy.
        InternalTaskOnly,
        /// Manager approval required outcome in the automation authority or human-review policy.
        ManagerApprovalRequired,
        /// Never automate outcome in the automation authority or human-review policy.
        NeverAutomate,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Automation policy rule tying a workflow name to an authority level and rationale.
    pub struct Rule {
        /// Policy input workflow used to explain or enforce an automation gate.
        pub workflow: WorkflowName,
        /// Policy input level used to explain or enforce an automation gate.
        pub level: Level,
        /// Policy input rationale used to explain or enforce an automation gate.
        pub rationale: Rationale,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Vaccine requirement for a species/service pair, including whether proof must come from a licensed vet source.
pub struct VaccineRequirement {
    /// Policy input species used to explain or enforce an automation gate.
    pub species: Species,
    /// Requested service that drives scheduling and labor estimates.
    pub service: ServiceKind,
    /// Policy input vaccines used to explain or enforce an automation gate.
    pub vaccines: Vec<VaccineName>,
    /// Policy input source must be licensed vet used to explain or enforce an automation gate.
    pub source_must_be_licensed_vet: bool,
}

/// Group-play eligibility policies used to protect pet safety while avoiding unnecessary manual triage.
pub mod play {
    pub use eligibility::{
        ConservativePolicy, Decision, Eligibility, IneligibilityReason, Policy, Reason,
    };

    /// Decision record for whether a pet/service combination may enter group-play workflows.
    pub mod eligibility {
        use serde::{Deserialize, Serialize};

        use crate::entities::{Pet, ServiceKind, SpayNeuterStatus, Species};

        use super::super::ReviewGate;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Group-play eligibility decision plus any review gate that must be satisfied first.
        pub struct Decision {
            /// Policy input eligibility used to explain or enforce an automation gate.
            pub eligibility: Eligibility,
            /// Policy input required review used to explain or enforce an automation gate.
            pub required_review: Option<ReviewGate>,
        }

        impl Decision {
            /// Reports whether the policy outcome allows the pet to be treated as a group-play candidate.
            pub fn eligible_for_group_play(&self) -> bool {
                matches!(self.eligibility, Eligibility::Eligible(_))
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Eligibility outcome produced by play-safety policy evaluation.
        pub enum Eligibility {
            /// Eligible outcome in the automation authority or human-review policy.
            Eligible(Reason),
            /// Ineligible outcome in the automation authority or human-review policy.
            Ineligible(IneligibilityReason),
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Positive policy reason explaining why no conservative hard stop blocked the workflow.
        pub enum Reason {
            /// No conservative hard stop outcome in the automation authority or human-review policy.
            NoConservativeHardStop,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        /// Safety or service reason that prevents group play or requires staff review.
        pub enum IneligibilityReason {
            /// Service does not require group play outcome in the automation authority or human-review policy.
            ServiceDoesNotRequireGroupPlay,
            /// Species receives individual play outcome in the automation authority or human-review policy.
            SpeciesReceivesIndividualPlay,
            /// Spay neuter status requires review outcome in the automation authority or human-review policy.
            SpayNeuterStatusRequiresReview,
            /// Behavior flags require review outcome in the automation authority or human-review policy.
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

        /// Policy evaluator that turns pet and service facts into explicit play-safety decisions.
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

/// Denial reasons that explain why a workflow is blocked or escalated to review.
pub mod denial {
    use serde::{Deserialize, Serialize};

    use super::play;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Positive policy reason explaining why no conservative hard stop blocked the workflow.
    pub enum Reason {
        /// Manager approval required outcome in the automation authority or human-review policy.
        ManagerApprovalRequired,
        /// Medical document review required outcome in the automation authority or human-review policy.
        MedicalDocumentReviewRequired,
        /// Behavior history requires review before service.
        BehaviorReviewRequired,
        /// Customer message approval required outcome in the automation authority or human-review policy.
        CustomerMessageApprovalRequired,
        /// Refund or deposit exception outcome in the automation authority or human-review policy.
        RefundOrDepositException,
        /// Play eligibility outcome in the automation authority or human-review policy.
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
/// Human review gate required before automation may proceed with sensitive work.
pub enum ReviewGate {
    /// Manager approval outcome in the automation authority or human-review policy.
    ManagerApproval,
    /// Medical document review outcome in the automation authority or human-review policy.
    MedicalDocumentReview,
    /// Behavior review outcome in the automation authority or human-review policy.
    BehaviorReview,
    /// Customer message approval outcome in the automation authority or human-review policy.
    CustomerMessageApproval,
    /// Refund or deposit exception outcome in the automation authority or human-review policy.
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
