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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Enrollment readiness state and the review gate that blocks assignment when data, behavior, care, or payment facts are incomplete.
pub enum Readiness {
    /// Enrollment has enough source facts to draft trainer assignment.
    Ready,
    /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
    TrainerReviewRequired {
        /// Approval gate staff must clear before acting on this variant.
        gate: policy::ReviewGate,
    },
    /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
    BehaviorOrCareReviewRequired {
        /// Approval gate staff must clear before acting on this variant.
        gate: policy::ReviewGate,
    },
    /// Review gate that must clear before this training decision affects assignment, package use, or parent-facing copy.
    PackageOrPaymentReviewRequired {
        /// Approval gate staff must clear before acting on this variant.
        gate: policy::ReviewGate,
    },
}

impl Readiness {}
