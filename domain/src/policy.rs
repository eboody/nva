//! Policy gates that decide what automation may do safely.
//!
//! Policy values encode the operating line between manual-work-reduction assistance and human review:
//! group-play eligibility, vaccine requirements, manager approval, medical-document review,
//! customer-message approval, and refund/deposit exceptions. These types document why an agent may
//! draft, route, suppress, or escalate work; they do not grant permission to override local resort
//! policy or invent availability.

use nutype::nutype;
use serde::{Deserialize, Serialize};

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

/// Automation-level policy rules that classify workflows as safe, draft-only, internal, or review-gated.
pub mod automation {
    use serde::{Deserialize, Serialize};

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
}

/// Group-play eligibility policies used to protect pet safety while avoiding unnecessary manual triage.
pub mod play {
    pub use eligibility::{ConservativePolicy, Decision, IneligibilityReason, Policy, Reason};

    /// Decision record for whether a pet/service combination may enter group-play workflows.
    pub mod eligibility {
        use serde::{Deserialize, Serialize};

        use crate::entities::{Pet, ServiceKind, SpayNeuterStatus, Species};

        use super::super::ReviewGate;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        /// Coherent group-play policy state.
        ///
        /// Each variant carries exactly the facts legal for that outcome, so persisted decisions
        /// cannot claim eligibility while simultaneously requiring review.
        pub enum Decision {
            /// No conservative hard stop blocks group play.
            Eligible {
                /// Positive reason supporting eligibility.
                reason: Reason,
            },
            /// A definitive policy condition blocks group play without a review path.
            Ineligible {
                /// Policy reason that blocks group play.
                reason: IneligibilityReason,
            },
            /// Available evidence is insufficient for eligibility until a named review occurs.
            ReviewRequired {
                /// Evidence condition that requires review.
                reason: IneligibilityReason,
                /// Exact review gate needed before a later policy decision.
                gate: ReviewGate,
            },
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
                    return Decision::Ineligible {
                        reason: IneligibilityReason::ServiceDoesNotRequireGroupPlay,
                    };
                }

                if pet.species != Species::Dog {
                    return Decision::Ineligible {
                        reason: IneligibilityReason::SpeciesReceivesIndividualPlay,
                    };
                }

                if matches!(
                    pet.spay_neuter_status,
                    SpayNeuterStatus::Intact | SpayNeuterStatus::Unknown
                ) {
                    return Decision::ReviewRequired {
                        reason: IneligibilityReason::SpayNeuterStatusRequiresReview,
                        gate: ReviewGate::BehaviorReview,
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
                    return Decision::ReviewRequired {
                        reason: IneligibilityReason::BehaviorFlagsRequireReview,
                        gate: ReviewGate::BehaviorReview,
                    };
                }

                Decision::Eligible {
                    reason: Reason::NoConservativeHardStop,
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
