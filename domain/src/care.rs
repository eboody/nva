use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::fmt;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct FeedingInstruction(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct AllergyName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct MedicalConditionName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct MedicalNote(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct ContactName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct MedicationName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct MedicationDose(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct MedicationSchedule(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
pub struct ReviewReason(String);

macro_rules! redacted_debug {
    ($type:ident, $label:literal) => {
        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str($label)
            }
        }
    };
}

redacted_debug!(FeedingInstruction, "FeedingInstruction(<redacted>)");
redacted_debug!(AllergyName, "AllergyName(<redacted>)");
redacted_debug!(MedicalConditionName, "MedicalConditionName(<redacted>)");
redacted_debug!(MedicalNote, "MedicalNote(<redacted>)");
redacted_debug!(ContactName, "ContactName(<redacted>)");
redacted_debug!(MedicationName, "MedicationName(<redacted>)");
redacted_debug!(MedicationDose, "MedicationDose(<redacted>)");
redacted_debug!(MedicationSchedule, "MedicationSchedule(<redacted>)");
redacted_debug!(ReviewReason, "ReviewReason(<redacted>)");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactRef {
    pub name: ContactName,
}

impl ContactRef {
    pub fn new(name: ContactName) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MedicationReviewRequirement {
    NotRequired,
    RequiresReview { reason: ReviewReason },
}

impl MedicationReviewRequirement {
    pub fn requires_review(&self) -> bool {
        matches!(self, Self::RequiresReview { .. })
    }
}
