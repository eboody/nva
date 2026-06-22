//! Care-plan and medical-instruction value objects for safe resort workflows.
//!
//! ## Operator-summary
//!
//! This module supports the staff queue that turns feeding instructions, allergies,
//! medical conditions, medication schedules, emergency contacts, and veterinarian
//! contacts into safe care tasks and shift handoffs. It can reduce labor by making
//! medication-administration work, special handling, and daily-brief warnings visible
//! without forcing staff to reread free-text pet notes for every stay.
//!
//! It must not automate live medical, medication, grooming, boarding, or customer
//! communication decisions. Medication names, doses, schedules, medical notes, allergy
//! labels, customer/provider instructions, and veterinarian or emergency-contact facts
//! remain authoritative only as their reviewed source records and approval history allow;
//! this module merely preserves those facts as redacted domain values. Review gates protect
//! pets, customers, and staff by requiring care-team review before ambiguous, sensitive,
//! or changed medication/special-care instructions can drive service work or
//! customer-visible copy.
//!
//! Care data is sensitive source evidence: these values promote provider/customer facts
//! into redacted, validated domain types before staff tasks, daily briefs, or customer
//! messaging can use them. Review requirements make medication and special-handling labor
//! explicit instead of hiding the work in free-text notes.

use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::fmt;

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted feeding instructions that can create care tasks and labor requirements.
pub struct FeedingInstruction(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 120),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted allergy label that guards unsafe care or grooming recommendations.
pub struct AllergyName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted medical-condition label requiring careful human handling.
pub struct MedicalConditionName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 1000),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted medical note retained as evidence, not as autonomous medical advice.
pub struct MedicalNote(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted staff/customer contact name for care-plan coordination.
pub struct ContactName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted medication name used to schedule administration work safely.
pub struct MedicationName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 160),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted medication dose retained for staff review and audit trails.
pub struct MedicationDose(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted medication schedule that can drive labor and shift-handoff tasks.
pub struct MedicationSchedule(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 400),
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
)]
/// Redacted reason explaining why care-team review is required before action.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Checkout care exception retained when care summary or departure-note evidence needs staff/manager review.
pub enum CheckoutException {
    /// Care summary or departure notes still need staff/manager review before checkout confidence.
    DepartureCareReviewRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Named staff or customer contact used for care-plan coordination.
pub struct ContactRef {
    /// Contact or display name used by staff.
    pub name: ContactName,
}

impl ContactRef {
    /// Assembles this care value from already-validated domain parts.
    pub fn new(name: ContactName) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Whether medication instructions require additional care-team review.
pub enum MedicationReviewRequirement {
    /// Medication instructions do not add a review gate beyond normal staff handling.
    NotRequired,
    /// Business reason staff should review before proceeding.
    RequiresReview {
        /// Care reason staff should review before applying the override.
        reason: ReviewReason,
    },
}

impl MedicationReviewRequirement {
    /// Returns whether care-team review is required before proceeding.
    pub fn requires_review(&self) -> bool {
        matches!(self, Self::RequiresReview { .. })
    }
}
