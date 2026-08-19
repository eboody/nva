//! Pet identity, temperament, and care-profile aggregates.

use bon::Builder;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::identifiers::{CustomerId, PetId};
use crate::{care, pet, temperament};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Pet profile carrying identity, species, age, sex, sterilization, temperament, and care facts for safe service decisions.
pub struct Pet {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: PetId,
    /// Customer id retained from source records for staff review, safety gates, and workflow joins.
    pub customer_id: CustomerId,
    /// Contact or display name used by staff.
    pub name: pet::Name,
    /// Species retained from source records for staff review, safety gates, and workflow joins.
    pub species: Species,
    /// Birth date retained from source records for staff review, safety gates, and workflow joins.
    pub birth_date: Option<NaiveDate>,
    /// Sex retained from source records for staff review, safety gates, and workflow joins.
    pub sex: Option<Sex>,
    /// Spay neuter status retained from source records for staff review, safety gates, and workflow joins.
    pub spay_neuter_status: SpayNeuterStatus,
    #[builder(default)]
    /// Temperament retained from source records for staff review, safety gates, and workflow joins.
    pub temperament: TemperamentProfile,
    #[builder(default)]
    /// Care profile retained from source records for staff review, safety gates, and workflow joins.
    pub care_profile: CareProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Pet species category used by boarding/daycare/play policies and labor planning.
pub enum Species {
    /// Dog guest, using dog-specific policy and capacity rules.
    Dog,
    /// Cat guest, using cat-specific policy and accommodation rules.
    Cat,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Recorded pet sex when the source system supplies it.
pub enum Sex {
    /// Female pet sex recorded for profile and policy context.
    Female,
    /// Male pet sex recorded for profile and policy context.
    Male,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Spay/neuter status used by group-play eligibility, safety review, and policy gating.
pub enum SpayNeuterStatus {
    /// Pet has been spayed for policy and playgroup eligibility checks.
    Spayed,
    /// Pet has been neutered for policy and playgroup eligibility checks.
    Neutered,
    /// Pet is intact and may trigger extra policy review.
    Intact,
    /// Provider role or status could not be mapped confidently.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder, Default)]
/// Temperament evidence used to decide group-play, individual care, and behavior-review routing.
pub struct TemperamentProfile {
    #[builder(default)]
    /// Group play observation retained from source records for staff review, safety gates, and workflow joins.
    pub group_play_observation: temperament::GroupPlayObservation,
    #[builder(default)]
    /// People orientation retained from source records for staff review, safety gates, and workflow joins.
    pub people_orientation: temperament::PeopleOrientation,
    #[builder(default)]
    /// Rating retained from source records for staff review, safety gates, and workflow joins.
    pub rating: temperament::Rating,
    #[builder(default)]
    /// Behavior observations retained from source records for staff review, safety gates, and workflow joins.
    pub behavior_observations: Vec<temperament::BehaviorObservation>,
    #[builder(default)]
    /// Staff notes retained from source records for staff review, safety gates, and workflow joins.
    pub staff_notes: Vec<temperament::StaffNote>,
}

impl TemperamentProfile {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Feeding, medication, handling, and special-care summary used for staff handoffs and briefings.
pub struct CareProfile {
    /// Feeding instructions retained from source records for staff review, safety gates, and workflow joins.
    pub feeding_instructions: Option<care::FeedingInstruction>,
    /// Medications retained from source records for staff review, safety gates, and workflow joins.
    pub medications: Vec<MedicationInstruction>,
    /// Allergies retained from source records for staff review, safety gates, and workflow joins.
    pub allergies: Vec<care::AllergyName>,
    /// Medical conditions retained from source records for staff review, safety gates, and workflow joins.
    pub medical_conditions: Vec<care::MedicalConditionName>,
    /// Emergency contact retained from source records for staff review, safety gates, and workflow joins.
    pub emergency_contact: Option<care::ContactRef>,
    /// Veterinarian contact retained from source records for staff review, safety gates, and workflow joins.
    pub veterinarian_contact: Option<care::ContactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Medication instruction that must remain explicit for care safety and shift handoff evidence.
pub struct MedicationInstruction {
    /// Contact or display name used by staff.
    pub name: care::MedicationName,
    /// Dose retained from source records for staff review, safety gates, and workflow joins.
    pub dose: care::MedicationDose,
    /// Schedule retained from source records for staff review, safety gates, and workflow joins.
    pub schedule: care::MedicationSchedule,
    /// Review requirement retained from source records for staff review, safety gates, and workflow joins.
    pub review_requirement: care::MedicationReviewRequirement,
}
