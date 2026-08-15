//! Core pet-resort entities and operational records.
//!
//! ## Operator-summary
//!
//! This module supports the shared staff view of pets, customers, reservations, care profiles,
//! documents, vaccine records, care notes, incidents, messages, and approval records. It can
//! reduce labor by keeping the facts needed for triage, safety review, handoff, document review,
//! customer-message approval, and manager queues in one normalized shape instead of scattering
//! them across source-system payloads and free-text notes.
//!
//! It must not automate live booking changes, provider writes, customer sends, payment/refund
//! actions, medical/vaccine/behavior decisions, incident closure, or policy exceptions.
//! Authoritative facts remain the named source record, source document/storage object, policy
//! snapshot, reviewer approval, audit event, and typed domain value for each field. Review
//! gates protect pets, customers, and staff by tying sensitive records to explicit approval
//! targets and lifecycle states before downstream workflows may treat them as cleared.
//!
//! These structs and enums are the normalized domain facts used by workflow, policy, storage, and
//! source adapters. They should be read as normalized operating records: every field is either a source-backed
//! fact, a reviewable derived state, or a safety/labor signal used to reduce manual resort work
//! without bypassing manager, medical, behavior, payment, or customer-message gates.

use chrono::{DateTime, NaiveDate, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use bon::Builder;

use crate::{
    agent, care, customer, document, incident, location, message, payment, pet, policy, portal,
    temperament, vaccine,
};

/// Error returned when a sentinel UUID is offered as a production domain identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("domain identity UUID cannot be nil")]
pub struct NilIdentityError;

macro_rules! non_nil_uuid_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Constructs an identity for trusted static/programmatic values and rejects nil.
            #[track_caller]
            pub fn new(value: Uuid) -> Self {
                Self::try_new(value).expect("domain identity UUID cannot be nil")
            }

            /// Promotes an untrusted UUID only when it is not the nil sentinel.
            pub const fn try_new(
                value: Uuid,
            ) -> std::result::Result<Self, $crate::entities::NilIdentityError> {
                if value.is_nil() {
                    Err($crate::entities::NilIdentityError)
                } else {
                    Ok(Self(value))
                }
            }

            /// Returns the validated UUID representation.
            pub const fn get(self) -> Uuid {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Uuid::deserialize(deserializer)?;
                Self::try_new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

non_nil_uuid_id!(
    LocationId,
    "Stable non-nil identifier for a resort location across source imports, policies, reports, and workflows."
);
non_nil_uuid_id!(
    CustomerId,
    "Stable non-nil identifier for the customer/account responsible for pets, reservations, messages, and payments."
);
non_nil_uuid_id!(
    PetId,
    "Stable non-nil identifier for a pet whose care, temperament, vaccine, and reservation facts drive safety decisions."
);

impl std::fmt::Display for PetId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Reservation-facing source vocabulary embedded in core entity records.
pub mod reservation {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeSet;
    use std::fmt;
    use uuid::Uuid;

    use super::{PetId, PortalProvider};

    non_nil_uuid_id!(
        Id,
        "Stable non-nil provider or source reservation identifier retained as a join key."
    );

    impl fmt::Display for Id {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(formatter)
        }
    }

    /// Reservation aggregate construction and rehydration failures.
    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        /// Builder did not receive the stable reservation id required for workflow/storage joins.
        #[error("reservation id is required")]
        IdRequired,
        /// Builder did not receive the owning location id required for policy and labor routing.
        #[error("reservation location id is required")]
        LocationIdRequired,
        /// Builder did not receive the customer id required to own the pet party.
        #[error("reservation customer id is required")]
        CustomerIdRequired,
        /// Builder did not receive the requested service line.
        #[error("reservation service is required")]
        ServiceRequired,
        /// Builder did not receive the normalized reservation status.
        #[error("reservation status is required")]
        StatusRequired,
        /// Builder did not receive the stay start instant.
        #[error("reservation start instant is required")]
        StartsAtRequired,
        /// Builder did not receive the stay end instant.
        #[error("reservation end instant is required")]
        EndsAtRequired,
        /// Builder did not receive the source channel for reservation evidence.
        #[error("reservation source is required")]
        SourceRequired,
        /// Stay interval ended at or before its start instant.
        #[error("reservation end must be after start")]
        StayIntervalMustEndAfterStart,
        /// Reservation party omitted every pet, which would make booking/care ownership meaningless.
        #[error("reservation requires at least one pet")]
        PetPartyRequired,
        /// Same pet appeared more than once in the reservation party.
        #[error("reservation pet party contains duplicate pet {pet_id}")]
        DuplicatePet {
            /// Pet id duplicated in the reservation party.
            pet_id: PetId,
        },
        /// Deposit-required hard stop was attached even though the deposit no longer needs collection.
        #[error("deposit-required hard stop requires a collectible deposit")]
        DepositRequiredHardStopNeedsCollectibleDeposit,
        /// Terminal reservations cannot keep active staff hard stops attached.
        #[error("terminal reservation status must not carry active hard stops")]
        TerminalReservationCannotCarryActiveHardStops,
    }

    /// Result alias for reservation aggregate construction and rehydration.
    pub type Result<T> = std::result::Result<T, Error>;

    /// Checked, non-empty, duplicate-free pet party for a reservation.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct PetParty(Vec<PetId>);

    impl PetParty {
        /// Validates the reservation party contains at least one distinct pet.
        pub fn try_new(pet_ids: Vec<PetId>) -> Result<Self> {
            if pet_ids.is_empty() {
                return Err(Error::PetPartyRequired);
            }
            let mut seen = BTreeSet::new();
            for pet_id in &pet_ids {
                if !seen.insert(*pet_id) {
                    return Err(Error::DuplicatePet { pet_id: *pet_id });
                }
            }
            Ok(Self(pet_ids))
        }

        /// Returns the ordered pet ids participating in the reservation.
        pub fn as_slice(&self) -> &[PetId] {
            &self.0
        }

        pub(super) fn into_vec(self) -> Vec<PetId> {
            self.0
        }
    }

    /// Checked reservation stay interval whose end is strictly after its start.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    pub struct StayInterval {
        starts_at: chrono::DateTime<chrono::Utc>,
        ends_at: chrono::DateTime<chrono::Utc>,
    }

    impl StayInterval {
        /// Validates that a reservation stay ends after it starts.
        pub fn try_new(
            starts_at: chrono::DateTime<chrono::Utc>,
            ends_at: chrono::DateTime<chrono::Utc>,
        ) -> Result<Self> {
            if ends_at <= starts_at {
                return Err(Error::StayIntervalMustEndAfterStart);
            }
            Ok(Self { starts_at, ends_at })
        }

        /// Reservation start instant.
        pub fn starts_at(&self) -> chrono::DateTime<chrono::Utc> {
            self.starts_at
        }

        /// Reservation end instant.
        pub fn ends_at(&self) -> chrono::DateTime<chrono::Utc> {
            self.ends_at
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized lifecycle states used to reconcile source-system data with domain workflows.
    pub enum Status {
        /// Inquiry state or source category preserved for normalized resort records.
        Inquiry,
        /// Reservation has been requested but not yet confirmed.
        Requested,
        /// Missing info state or source category preserved for normalized resort records.
        MissingInfo,
        /// Vaccine pending state or source category preserved for normalized resort records.
        VaccinePending,
        /// Special review state or source category preserved for normalized resort records.
        SpecialReview,
        /// Waitlisted state or source category preserved for normalized resort records.
        Waitlisted,
        /// Offered state or source category preserved for normalized resort records.
        Offered,
        /// Reservation has been accepted by the resort.
        Confirmed,
        /// Pet has arrived and is in care.
        CheckedIn,
        /// Active state or source category preserved for normalized resort records.
        Active,
        /// Pet has left care and the stay is complete.
        CheckedOut,
        /// Reservation is no longer active.
        Cancelled,
        /// Rejected state or source category preserved for normalized resort records.
        Rejected,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Origin channel for a reservation or operational fact before it becomes trusted domain evidence.
    pub enum Source {
        /// Portal state or source category preserved for normalized resort records.
        Portal(PortalProvider),
        /// Website form state or source category preserved for normalized resort records.
        WebsiteForm,
        /// Phone transcript state or source category preserved for normalized resort records.
        PhoneTranscript,
        /// Sms state or source category preserved for normalized resort records.
        Sms,
        /// Email state or source category preserved for normalized resort records.
        Email,
        /// Staff created state or source category preserved for normalized resort records.
        StaffCreated,
    }
}

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
pub struct StaffId(String);

/// Manager identifier used when approvals, overrides, or escalations require accountable leadership.
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
pub struct ManagerId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Resort location record that scopes local capabilities, timezone, brand, and policy references.
pub struct Location {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: LocationId,
    /// Brand retained from source records for staff review, safety gates, and workflow joins.
    pub brand: Brand,
    /// Contact or display name used by staff.
    pub name: location::Name,
    /// Timezone retained from source records for staff review, safety gates, and workflow joins.
    pub timezone: location::Timezone,
    /// Capabilities retained from source records for staff review, safety gates, and workflow joins.
    pub capabilities: Vec<ServiceKind>,
    /// Policies retained from source records for staff review, safety gates, and workflow joins.
    pub policies: LocationPolicyRefs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Brand family used to group multi-site operating records without losing local resort identity.
pub enum Brand {
    /// Nva pet resorts state or source category preserved for normalized resort records.
    NvaPetResorts,
    /// Pet suites state or source category preserved for normalized resort records.
    PetSuites,
    /// Contact or display name used by staff.
    NeighborhoodPetResort {
        /// Name attached to this variant for reviewers and adapters.
        name: location::Name,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// References to the local policy set that controls automation, vaccine, and play-safety decisions.
pub struct LocationPolicyRefs {
    /// Vaccine policy id retained from source records for staff review, safety gates, and workflow joins.
    pub vaccine_policy_id: policy::Id,
    /// Deposit policy id retained from source records for staff review, safety gates, and workflow joins.
    pub deposit_policy_id: policy::Id,
    /// Playgroup policy id retained from source records for staff review, safety gates, and workflow joins.
    pub playgroup_policy_id: policy::Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Customer/account profile used for reservation ownership, consent-sensitive messaging, and follow-up work.
pub struct Customer {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: CustomerId,
    /// Full name retained from source records for staff review, safety gates, and workflow joins.
    pub full_name: customer::Name,
    /// Email retained from source records for staff review, safety gates, and workflow joins.
    pub email: Option<customer::Email>,
    /// Mobile phone retained from source records for staff review, safety gates, and workflow joins.
    pub mobile_phone: Option<customer::Phone>,
    /// Preferred contact retained from source records for staff review, safety gates, and workflow joins.
    pub preferred_contact: ContactChannel,
    /// Portal account retained from source records for staff review, safety gates, and workflow joins.
    pub portal_account: Option<PortalAccountRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Link to the customer portal account that supplied or owns source records.
pub struct PortalAccountRef {
    /// Provider retained from source records for staff review, safety gates, and workflow joins.
    pub provider: PortalProvider,
    /// External customer id retained from source records for staff review, safety gates, and workflow joins.
    pub external_customer_id: portal::CustomerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Portal provider that owns the account or operational record.
pub enum PortalProvider {
    /// Gingr reservation and pet-care operating system.
    Gingr,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Customer contact channel preference or observed route used by draft/message workflows.
pub enum ContactChannel {
    /// Email state or source category preserved for normalized resort records.
    Email,
    /// Sms state or source category preserved for normalized resort records.
    Sms,
    /// Phone state or source category preserved for normalized resort records.
    Phone,
    /// Portal state or source category preserved for normalized resort records.
    Portal,
}

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

impl TemperamentProfile {
    /// Reports whether temperament facts require staff evaluation before group play or similar services.
    pub fn needs_staff_play_evaluation(&self) -> bool {
        self.group_play_observation.needs_staff_evaluation()
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Reservation record tying customer, pet, service, status, deposit, add-ons, and safety stops together.
pub struct Reservation {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    id: reservation::Id,
    /// Location id retained from source records for staff review, safety gates, and workflow joins.
    location_id: LocationId,
    /// Customer id retained from source records for staff review, safety gates, and workflow joins.
    customer_id: CustomerId,
    /// Pet ids retained from source records for staff review, safety gates, and workflow joins.
    pet_party: reservation::PetParty,
    /// Requested service that drives scheduling and labor estimates.
    service: ServiceKind,
    /// Status retained from source records for staff review, safety gates, and workflow joins.
    status: reservation::Status,
    /// Starts at retained from source records for staff review, safety gates, and workflow joins.
    stay_interval: reservation::StayInterval,
    /// Ends at retained from source records for staff review, safety gates, and workflow joins.
    deposit: Option<Deposit>,
    /// Source retained from source records for staff review, safety gates, and workflow joins.
    source: reservation::Source,
    /// Requested add ons retained from source records for staff review, safety gates, and workflow joins.
    requested_add_ons: Vec<AddOn>,
    /// Hard stops retained from source records for staff review, safety gates, and workflow joins.
    hard_stops: Vec<HardStop>,
}

#[derive(Serialize, Deserialize)]
struct RawReservation {
    id: reservation::Id,
    location_id: LocationId,
    customer_id: CustomerId,
    pet_ids: Vec<PetId>,
    service: ServiceKind,
    status: reservation::Status,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    deposit: Option<Deposit>,
    source: reservation::Source,
    #[serde(default)]
    requested_add_ons: Vec<AddOn>,
    #[serde(default)]
    hard_stops: Vec<HardStop>,
}

impl Serialize for Reservation {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        RawReservation {
            id: self.id,
            location_id: self.location_id,
            customer_id: self.customer_id,
            pet_ids: self.pet_party.clone().into_vec(),
            service: self.service.clone(),
            status: self.status.clone(),
            starts_at: self.starts_at(),
            ends_at: self.ends_at(),
            deposit: self.deposit.clone(),
            source: self.source.clone(),
            requested_add_ons: self.requested_add_ons.clone(),
            hard_stops: self.hard_stops.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Reservation {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawReservation::deserialize(deserializer)?;
        Self::try_from_persisted(raw).map_err(serde::de::Error::custom)
    }
}

impl Reservation {
    fn try_from_persisted(raw: RawReservation) -> reservation::Result<Self> {
        let stay_interval = reservation::StayInterval::try_new(raw.starts_at, raw.ends_at)?;
        let pet_party = reservation::PetParty::try_new(raw.pet_ids)?;
        if raw.hard_stops.contains(&HardStop::DepositRequired)
            && !raw
                .deposit
                .as_ref()
                .is_some_and(payment::Deposit::requires_collection)
        {
            return Err(reservation::Error::DepositRequiredHardStopNeedsCollectibleDeposit);
        }
        if matches!(
            raw.status,
            reservation::Status::Cancelled
                | reservation::Status::Rejected
                | reservation::Status::CheckedOut
        ) && !raw.hard_stops.is_empty()
        {
            return Err(reservation::Error::TerminalReservationCannotCarryActiveHardStops);
        }

        Ok(Self {
            id: raw.id,
            location_id: raw.location_id,
            customer_id: raw.customer_id,
            pet_party,
            service: raw.service,
            status: raw.status,
            stay_interval,
            deposit: raw.deposit,
            source: raw.source,
            requested_add_ons: raw.requested_add_ons,
            hard_stops: raw.hard_stops,
        })
    }

    /// Starts a checked reservation aggregate builder.
    pub fn builder() -> ReservationBuilder {
        ReservationBuilder::default()
    }

    /// Reservation identifier used by workflow, storage, and review joins.
    pub fn id(&self) -> reservation::Id {
        self.id
    }

    /// Location that owns this reservation workflow.
    pub fn location_id(&self) -> LocationId {
        self.location_id
    }

    /// Customer/account responsible for the reservation party.
    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }

    /// Ordered, duplicate-free pet party.
    pub fn pet_ids(&self) -> &[PetId] {
        self.pet_party.as_slice()
    }

    /// Requested service line for labor planning and policy review.
    pub fn service(&self) -> &ServiceKind {
        &self.service
    }

    /// Current normalized reservation status.
    pub fn status(&self) -> &reservation::Status {
        &self.status
    }

    /// Checked reservation stay interval.
    pub fn stay_interval(&self) -> reservation::StayInterval {
        self.stay_interval
    }

    /// Reservation start instant.
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.stay_interval.starts_at()
    }

    /// Reservation end instant.
    pub fn ends_at(&self) -> DateTime<Utc> {
        self.stay_interval.ends_at()
    }

    /// Deposit evidence attached to this reservation, if any.
    pub fn deposit(&self) -> Option<&Deposit> {
        self.deposit.as_ref()
    }

    /// Source channel for the reservation evidence.
    pub fn source(&self) -> &reservation::Source {
        &self.source
    }

    /// Requested add-ons that affect labor or care planning.
    pub fn requested_add_ons(&self) -> &[AddOn] {
        &self.requested_add_ons
    }

    /// Active hard stops requiring staff, manager, or policy review.
    pub fn hard_stops(&self) -> &[HardStop] {
        &self.hard_stops
    }
}

/// Builder for checked reservation aggregates.
#[derive(Debug, Clone, Default)]
pub struct ReservationBuilder {
    id: Option<reservation::Id>,
    location_id: Option<LocationId>,
    customer_id: Option<CustomerId>,
    pet_ids: Vec<PetId>,
    service: Option<ServiceKind>,
    status: Option<reservation::Status>,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
    deposit: Option<Deposit>,
    source: Option<reservation::Source>,
    requested_add_ons: Vec<AddOn>,
    hard_stops: Vec<HardStop>,
}

impl ReservationBuilder {
    /// Sets the reservation id.
    pub fn id(mut self, id: reservation::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the owning location.
    pub fn location_id(mut self, location_id: LocationId) -> Self {
        self.location_id = Some(location_id);
        self
    }

    /// Sets the responsible customer.
    pub fn customer_id(mut self, customer_id: CustomerId) -> Self {
        self.customer_id = Some(customer_id);
        self
    }

    /// Adds one pet to the reservation party.
    pub fn pet_id(mut self, pet_id: PetId) -> Self {
        self.pet_ids.push(pet_id);
        self
    }

    /// Replaces the reservation pet party.
    pub fn pet_ids(mut self, pet_ids: Vec<PetId>) -> Self {
        self.pet_ids = pet_ids;
        self
    }

    /// Sets the requested service line.
    pub fn service(mut self, service: ServiceKind) -> Self {
        self.service = Some(service);
        self
    }

    /// Sets the reservation status.
    pub fn status(mut self, status: reservation::Status) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the reservation start instant.
    pub fn starts_at(mut self, starts_at: DateTime<Utc>) -> Self {
        self.starts_at = Some(starts_at);
        self
    }

    /// Sets the reservation end instant.
    pub fn ends_at(mut self, ends_at: DateTime<Utc>) -> Self {
        self.ends_at = Some(ends_at);
        self
    }

    /// Sets deposit evidence for this reservation.
    pub fn deposit(mut self, deposit: Deposit) -> Self {
        self.deposit = Some(deposit);
        self
    }

    /// Sets the source channel for the reservation evidence.
    pub fn source(mut self, source: reservation::Source) -> Self {
        self.source = Some(source);
        self
    }

    /// Adds one requested add-on.
    pub fn requested_add_on(mut self, requested_add_on: AddOn) -> Self {
        self.requested_add_ons.push(requested_add_on);
        self
    }

    /// Replaces requested add-ons.
    pub fn requested_add_ons(mut self, requested_add_ons: Vec<AddOn>) -> Self {
        self.requested_add_ons = requested_add_ons;
        self
    }

    /// Adds one active hard stop.
    pub fn hard_stop(mut self, hard_stop: HardStop) -> Self {
        self.hard_stops.push(hard_stop);
        self
    }

    /// Replaces active hard stops.
    pub fn hard_stops(mut self, hard_stops: Vec<HardStop>) -> Self {
        self.hard_stops = hard_stops;
        self
    }

    /// Builds a reservation only after all aggregate invariants pass.
    pub fn build(self) -> reservation::Result<Reservation> {
        Reservation::try_from_persisted(RawReservation {
            id: self.id.ok_or(reservation::Error::IdRequired)?,
            location_id: self
                .location_id
                .ok_or(reservation::Error::LocationIdRequired)?,
            customer_id: self
                .customer_id
                .ok_or(reservation::Error::CustomerIdRequired)?,
            pet_ids: self.pet_ids,
            service: self.service.ok_or(reservation::Error::ServiceRequired)?,
            status: self.status.ok_or(reservation::Error::StatusRequired)?,
            starts_at: self.starts_at.ok_or(reservation::Error::StartsAtRequired)?,
            ends_at: self.ends_at.ok_or(reservation::Error::EndsAtRequired)?,
            deposit: self.deposit,
            source: self.source.ok_or(reservation::Error::SourceRequired)?,
            requested_add_ons: self.requested_add_ons,
            hard_stops: self.hard_stops,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Resort service line used for labor planning, capacity, policy, upsell, and workflow routing.
pub enum ServiceKind {
    /// Overnight stay service line.
    Boarding,
    /// Single-day play visit without overnight lodging.
    DayPlay,
    /// Daytime boarding care with lodging-style supervision.
    DayBoarding,
    /// Grooming service line or care-note category.
    Grooming,
    /// Training service line or care-note category.
    Training,
    /// Day-spa service package.
    DaySpa,
}

/// Shared deposit type used by reservation, payment, and approval records.
pub type Deposit = payment::Deposit;
/// Shared payment status used by checkout, deposit, refund, and approval records.
pub type PaymentStatus = payment::DepositStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Optional reservation add-ons that affect labor, revenue, care planning, or customer follow-up.
pub enum AddOn {
    /// Group-play add-on or accommodation feature.
    GroupPlay,
    /// Individual play add-on for pets not suited to group play.
    IndividualPlay,
    /// Premium suite with webcam visibility.
    WebcamSuite,
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Progress report shared with the customer during care.
    PawgressReport,
    /// Medication service that requires care instructions.
    MedicationAdministration,
    /// Non-dog, non-cat pet handled by exception policy.
    Other(crate::reservation::AddOnLabel),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Non-ignorable condition that blocks or routes a reservation before staff or customer action proceeds.
pub enum HardStop {
    /// Missing required vaccine state or source category preserved for normalized resort records.
    MissingRequiredVaccine(policy::VaccineName),
    /// Ineligible for group play state or source category preserved for normalized resort records.
    IneligibleForGroupPlay(policy::play::IneligibilityReason),
    /// Pet is in heat and requires policy handling.
    InHeat,
    /// Age below minimum weeks state or source category preserved for normalized resort records.
    AgeBelowMinimumWeeks(crate::reservation::AgeThreshold),
    /// Medical or medication information requires review before service.
    MedicalOrMedicationReviewRequired,
    /// Behavior history requires review before service.
    BehaviorReviewRequired,
    /// Deposit must be collected before the booking is secure.
    DepositRequired,
}

non_nil_uuid_id!(
    DocumentId,
    "Stable non-nil identifier for a document artifact used as vaccine, waiver, medical, or incident evidence."
);
non_nil_uuid_id!(
    VaccineRecordId,
    "Stable non-nil identifier for a vaccine compliance record tied to a pet and proof document."
);

/// Care-note vocabulary for staff-visible, customer-visible, and internal handoff notes.
pub mod care_note {
    use nutype::nutype;
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::{IncidentId, PetId, reservation};

    non_nil_uuid_id!(
        Id,
        "Stable non-nil provider or source reservation identifier retained as a join key."
    );

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Subject that a care, document, incident, audit, or message record is about.
    pub enum Subject {
        /// Pet record participating in the workflow.
        Pet(PetId),
        /// Reservation record participating in the workflow.
        Reservation(reservation::Id),
        /// Incident record participating in the workflow.
        Incident(IncidentId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Care-note category used to route safety, feeding, medication, behavior, and staff handoff information.
    pub enum Kind {
        /// Feeding state or source category preserved for normalized resort records.
        Feeding,
        /// Medication state or source category preserved for normalized resort records.
        Medication,
        /// Medical state or source category preserved for normalized resort records.
        Medical,
        /// Behavior state or source category preserved for normalized resort records.
        Behavior,
        /// Grooming service line or care-note category.
        Grooming,
        /// Training service line or care-note category.
        Training,
        /// General state or source category preserved for normalized resort records.
        General,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Visibility rule that determines whether a care note may be shown to customers or only staff.
    pub enum Visibility {
        /// Internal only state or source category preserved for normalized resort records.
        InternalOnly,
        /// Customer visible state or source category preserved for normalized resort records.
        CustomerVisible,
        /// Customer visible after review state or source category preserved for normalized resort records.
        CustomerVisibleAfterReview,
    }

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 2000),
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
    pub struct Body(String);
}

non_nil_uuid_id!(
    IncidentId,
    "Stable non-nil identifier for a pet, customer, or operational incident requiring evidence and follow-up."
);
non_nil_uuid_id!(
    MessageId,
    "Stable non-nil identifier for a customer or internal message workflow."
);

/// Approval record vocabulary for review-gated automation outcomes.
pub mod approval {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize};
    use uuid::Uuid;

    use super::{
        ActorRef, DocumentId, IncidentId, MessageId, VaccineRecordId, policy, reservation,
    };

    non_nil_uuid_id!(
        Id,
        "Stable non-nil provider or source reservation identifier retained as a join key."
    );

    /// Approval aggregate construction and rehydration failures.
    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        #[error("approval id is required")]
        /// Represents the `IdRequired` semantic case.
        IdRequired,
        #[error("approval target is required")]
        /// Represents the `TargetRequired` semantic case.
        TargetRequired,
        #[error("approval review gate is required")]
        /// Represents the `GateRequired` semantic case.
        GateRequired,
        #[error("approval lifecycle is required")]
        /// Represents the `LifecycleRequired` semantic case.
        LifecycleRequired,
        #[error("approval requester is required")]
        /// Represents the `RequestedByRequired` semantic case.
        RequestedByRequired,
        #[error("approval request time is required")]
        /// Represents the `RequestedAtRequired` semantic case.
        RequestedAtRequired,
        #[error("approval decision time cannot precede request time")]
        /// Represents the `DecisionTimePrecedesRequest` semantic case.
        DecisionTimePrecedesRequest,
        #[error("approval review gate does not match approval target")]
        /// Represents the `GateTargetMismatch` semantic case.
        GateTargetMismatch,
    }

    /// Result alias for approval aggregate construction and rehydration.
    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Approval record showing who decided, what target was reviewed, and what lifecycle state resulted.
    pub struct Record {
        /// Id retained from source records for staff review, safety gates, and workflow joins.
        id: Id,
        /// Target retained from source records for staff review, safety gates, and workflow joins.
        target: Target,
        /// Gate retained from source records for staff review, safety gates, and workflow joins.
        gate: policy::ReviewGate,
        /// Lifecycle retained from source records for staff review, safety gates, and workflow joins.
        lifecycle: Lifecycle,
        /// Requested by retained from source records for staff review, safety gates, and workflow joins.
        requested_by: ActorRef,
        /// Requested at retained from source records for staff review, safety gates, and workflow joins.
        requested_at: DateTime<Utc>,
        /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
        audit_refs: Vec<crate::audit::EventId>,
    }

    #[derive(Serialize, Deserialize)]
    struct RawRecord {
        id: Id,
        target: Target,
        gate: policy::ReviewGate,
        lifecycle: Lifecycle,
        requested_by: ActorRef,
        requested_at: DateTime<Utc>,
        #[serde(default)]
        audit_refs: Vec<crate::audit::EventId>,
    }

    impl RawRecord {
        fn try_into_record(self) -> Result<Record> {
            validate_gate_target(&self.gate, &self.target)?;
            if self
                .lifecycle
                .decision_actor_and_time()
                .is_some_and(|(_, decided_at)| decided_at < self.requested_at)
            {
                return Err(Error::DecisionTimePrecedesRequest);
            }
            Ok(Record {
                id: self.id,
                target: self.target,
                gate: self.gate,
                lifecycle: self.lifecycle,
                requested_by: self.requested_by,
                requested_at: self.requested_at,
                audit_refs: self.audit_refs,
            })
        }
    }

    impl<'de> Deserialize<'de> for Record {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            RawRecord::deserialize(deserializer)?
                .try_into_record()
                .map_err(serde::de::Error::custom)
        }
    }

    impl Record {
        /// Starts a checked approval aggregate builder.
        pub fn builder() -> RecordBuilder {
            RecordBuilder::default()
        }

        /// Approval id used by audit, storage, and authority evidence.
        pub fn id(&self) -> Id {
            self.id
        }

        /// Review target this approval applies to.
        pub fn target(&self) -> &Target {
            &self.target
        }

        /// Review gate this approval applies to.
        pub fn gate(&self) -> &policy::ReviewGate {
            &self.gate
        }

        /// Lifecycle state carried by this approval record.
        pub fn lifecycle(&self) -> &Lifecycle {
            &self.lifecycle
        }

        /// Actor that requested this approval.
        pub fn requested_by(&self) -> &ActorRef {
            &self.requested_by
        }

        /// Timestamp when approval was requested.
        pub fn requested_at(&self) -> DateTime<Utc> {
            self.requested_at
        }

        /// Audit refs attached to this approval.
        pub fn audit_refs(&self) -> &[crate::audit::EventId] {
            &self.audit_refs
        }

        /// Returns the normalized operational status represented by this record.
        pub fn status(&self) -> Status {
            self.lifecycle.status()
        }

        /// Reports whether this approval gate currently applies to the target workflow.
        pub fn is_applicable(&self) -> bool {
            matches!(self.lifecycle, Lifecycle::Approved { .. })
        }

        /// Reports whether the review lifecycle has reached an approval, rejection, or non-applicable endpoint.
        pub fn is_terminal_decision(&self) -> bool {
            self.lifecycle.is_terminal_decision()
        }

        /// Returns the accountable actor and timestamp when the review reached a terminal decision.
        pub fn decision_actor_and_time(&self) -> Option<(&ActorRef, DateTime<Utc>)> {
            self.lifecycle.decision_actor_and_time()
        }
    }

    /// Builder for checked approval aggregates.
    #[derive(Debug, Clone, Default)]
    pub struct RecordBuilder {
        id: Option<Id>,
        target: Option<Target>,
        gate: Option<policy::ReviewGate>,
        lifecycle: Option<Lifecycle>,
        requested_by: Option<ActorRef>,
        requested_at: Option<DateTime<Utc>>,
        audit_refs: Vec<crate::audit::EventId>,
    }

    impl RecordBuilder {
        /// Returns the aggregate id.
        pub fn id(mut self, id: Id) -> Self {
            self.id = Some(id);
            self
        }
        /// Returns the aggregate target.
        pub fn target(mut self, target: Target) -> Self {
            self.target = Some(target);
            self
        }
        /// Returns the aggregate gate.
        pub fn gate(mut self, gate: policy::ReviewGate) -> Self {
            self.gate = Some(gate);
            self
        }
        /// Returns the aggregate lifecycle.
        pub fn lifecycle(mut self, lifecycle: Lifecycle) -> Self {
            self.lifecycle = Some(lifecycle);
            self
        }
        /// Returns the aggregate requested by.
        pub fn requested_by(mut self, requested_by: ActorRef) -> Self {
            self.requested_by = Some(requested_by);
            self
        }
        /// Returns the aggregate requested at.
        pub fn requested_at(mut self, requested_at: DateTime<Utc>) -> Self {
            self.requested_at = Some(requested_at);
            self
        }
        /// Returns the aggregate audit refs.
        pub fn audit_refs(mut self, audit_refs: Vec<crate::audit::EventId>) -> Self {
            self.audit_refs = audit_refs;
            self
        }
        /// Validates the accumulated fields and builds the aggregate.
        pub fn build(self) -> Result<Record> {
            RawRecord {
                id: self.id.ok_or(Error::IdRequired)?,
                target: self.target.ok_or(Error::TargetRequired)?,
                gate: self.gate.ok_or(Error::GateRequired)?,
                lifecycle: self.lifecycle.ok_or(Error::LifecycleRequired)?,
                requested_by: self.requested_by.ok_or(Error::RequestedByRequired)?,
                requested_at: self.requested_at.ok_or(Error::RequestedAtRequired)?,
                audit_refs: self.audit_refs,
            }
            .try_into_record()
        }
    }

    fn validate_gate_target(gate: &policy::ReviewGate, target: &Target) -> Result<()> {
        let legal = matches!(
            (gate, target),
            (
                policy::ReviewGate::CustomerMessageApproval,
                Target::Message(_)
            ) | (
                policy::ReviewGate::MedicalDocumentReview,
                Target::Document(_)
            ) | (
                policy::ReviewGate::MedicalDocumentReview,
                Target::VaccineRecord(_)
            ) | (policy::ReviewGate::ManagerApproval, Target::Reservation(_))
                | (policy::ReviewGate::ManagerApproval, Target::Incident(_))
                | (
                    policy::ReviewGate::RefundOrDepositException,
                    Target::Reservation(_)
                )
                | (policy::ReviewGate::BehaviorReview, Target::Reservation(_))
                | (policy::ReviewGate::BehaviorReview, Target::Incident(_))
        );
        if legal {
            Ok(())
        } else {
            Err(Error::GateTargetMismatch)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Operational artifact that an approval gate is allowed to approve, reject, or mark non-applicable.
    pub enum Target {
        /// Reservation record participating in the workflow.
        Reservation(reservation::Id),
        /// Customer or pet document participating in review.
        Document(DocumentId),
        /// Vaccination document or status record under review.
        VaccineRecord(VaccineRecordId),
        /// Incident record participating in the workflow.
        Incident(IncidentId),
        /// Customer communication record participating in approval.
        Message(MessageId),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Approval lifecycle state for draft, requested, approved, rejected, or non-applicable review gates.
    pub enum Lifecycle {
        /// Approval requested state or source category preserved for normalized resort records.
        ApprovalRequested,
        /// Approved state or source category preserved for normalized resort records.
        Approved {
            /// Decided by retained from source records for staff review, safety gates, and workflow joins.
            decided_by: ActorRef,
            /// Decided at retained from source records for staff review, safety gates, and workflow joins.
            decided_at: DateTime<Utc>,
        },
        /// Rejected state or source category preserved for normalized resort records.
        Rejected {
            /// Decided by retained from source records for staff review, safety gates, and workflow joins.
            decided_by: ActorRef,
            /// Decided at retained from source records for staff review, safety gates, and workflow joins.
            decided_at: DateTime<Utc>,
        },
        /// Reservation is no longer active.
        Cancelled,
        /// Superseded state or source category preserved for normalized resort records.
        Superseded,
    }

    impl Lifecycle {
        /// Returns the normalized operational status represented by this record.
        pub fn status(&self) -> Status {
            match self {
                Self::ApprovalRequested => Status::ApprovalRequested,
                Self::Approved { .. } => Status::Approved,
                Self::Rejected { .. } => Status::Rejected,
                Self::Cancelled => Status::Cancelled,
                Self::Superseded => Status::Superseded,
            }
        }

        /// Reports whether the review lifecycle has reached an approval, rejection, or non-applicable endpoint.
        pub fn is_terminal_decision(&self) -> bool {
            matches!(self, Self::Approved { .. } | Self::Rejected { .. })
        }

        /// Returns the accountable actor and timestamp when the review reached a terminal decision.
        pub fn decision_actor_and_time(&self) -> Option<(&ActorRef, DateTime<Utc>)> {
            match self {
                Self::Approved {
                    decided_by,
                    decided_at,
                }
                | Self::Rejected {
                    decided_by,
                    decided_at,
                } => Some((decided_by, *decided_at)),
                Self::ApprovalRequested | Self::Cancelled | Self::Superseded => None,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized lifecycle states used to reconcile source-system data with domain workflows.
    pub enum Status {
        /// Approval requested state or source category preserved for normalized resort records.
        ApprovalRequested,
        /// Approved state or source category preserved for normalized resort records.
        Approved,
        /// Rejected state or source category preserved for normalized resort records.
        Rejected,
        /// Reservation is no longer active.
        Cancelled,
        /// Superseded state or source category preserved for normalized resort records.
        Superseded,
    }
}

/// Document aggregate construction and rehydration failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error("document id is required")]
    /// Represents the `IdRequired` semantic case.
    IdRequired,
    #[error("document location id is required")]
    /// Represents the `LocationIdRequired` semantic case.
    LocationIdRequired,
    #[error("document subject is required")]
    /// Represents the `SubjectRequired` semantic case.
    SubjectRequired,
    #[error("document classification is required")]
    /// Represents the `ClassificationRequired` semantic case.
    ClassificationRequired,
    #[error("document source is required")]
    /// Represents the `SourceRequired` semantic case.
    SourceRequired,
    #[error("document uploader is required")]
    /// Represents the `UploadedByActorRequired` semantic case.
    UploadedByActorRequired,
    #[error("document upload time is required")]
    /// Represents the `UploadedAtRequired` semantic case.
    UploadedAtRequired,
    #[error("document original file evidence is required")]
    /// Represents the `OriginalFileRequired` semantic case.
    OriginalFileRequired,
    #[error("document storage reference is required")]
    /// Represents the `StorageRefRequired` semantic case.
    StorageRefRequired,
    #[error("document virus scan status is required")]
    /// Represents the `VirusScanStatusRequired` semantic case.
    VirusScanStatusRequired,
    #[error("document PII redaction status is required")]
    /// Represents the `PiiRedactionStatusRequired` semantic case.
    PiiRedactionStatusRequired,
    #[error("document verification status is required")]
    /// Represents the `VerificationStatusRequired` semantic case.
    VerificationStatusRequired,
    #[error("verified document requires passed virus scan")]
    /// Represents the `VerifiedRequiresPassedVirusScan` semantic case.
    VerifiedRequiresPassedVirusScan,
    #[error("verified document requires safe PII redaction status")]
    /// Represents the `VerifiedRequiresSafePiiRedactionStatus` semantic case.
    VerifiedRequiresSafePiiRedactionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Document record tying storage, classification, source, scan, redaction, and review status together.
pub struct Document {
    id: DocumentId,
    location_id: LocationId,
    subject: DocumentSubject,
    classification: document::Classification,
    source: document::Source,
    uploaded_by_actor: ActorRef,
    uploaded_at: DateTime<Utc>,
    original_file: document::OriginalFile,
    storage_ref: document::StorageRef,
    virus_scan_status: document::VirusScanStatus,
    pii_redaction_status: document::PiiRedactionStatus,
    verification_status: document::Status,
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawDocument {
    id: DocumentId,
    location_id: LocationId,
    subject: DocumentSubject,
    classification: document::Classification,
    source: document::Source,
    uploaded_by_actor: ActorRef,
    uploaded_at: DateTime<Utc>,
    original_file: document::OriginalFile,
    storage_ref: document::StorageRef,
    virus_scan_status: document::VirusScanStatus,
    pii_redaction_status: document::PiiRedactionStatus,
    verification_status: document::Status,
    #[serde(default)]
    audit_refs: Vec<crate::audit::EventId>,
}

impl RawDocument {
    fn try_into_document(self) -> std::result::Result<Document, DocumentError> {
        if matches!(self.verification_status, document::Status::Verified)
            && self.virus_scan_status != document::VirusScanStatus::Passed
        {
            return Err(DocumentError::VerifiedRequiresPassedVirusScan);
        }
        if matches!(self.verification_status, document::Status::Verified)
            && !matches!(
                self.pii_redaction_status,
                document::PiiRedactionStatus::NotRequired | document::PiiRedactionStatus::Redacted
            )
        {
            return Err(DocumentError::VerifiedRequiresSafePiiRedactionStatus);
        }
        Ok(Document {
            id: self.id,
            location_id: self.location_id,
            subject: self.subject,
            classification: self.classification,
            source: self.source,
            uploaded_by_actor: self.uploaded_by_actor,
            uploaded_at: self.uploaded_at,
            original_file: self.original_file,
            storage_ref: self.storage_ref,
            virus_scan_status: self.virus_scan_status,
            pii_redaction_status: self.pii_redaction_status,
            verification_status: self.verification_status,
            audit_refs: self.audit_refs,
        })
    }
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawDocument::deserialize(deserializer)?
            .try_into_document()
            .map_err(serde::de::Error::custom)
    }
}

impl Document {
    /// Starts checked construction of the aggregate.
    pub fn builder() -> DocumentBuilder {
        DocumentBuilder::default()
    }
    /// Returns the aggregate id.
    pub fn id(&self) -> DocumentId {
        self.id
    }
    /// Returns the aggregate location id.
    pub fn location_id(&self) -> LocationId {
        self.location_id
    }
    /// Returns the aggregate subject.
    pub fn subject(&self) -> &DocumentSubject {
        &self.subject
    }
    /// Returns the aggregate classification.
    pub fn classification(&self) -> document::Classification {
        self.classification
    }
    /// Returns the aggregate source.
    pub fn source(&self) -> document::Source {
        self.source
    }
    /// Returns the aggregate uploaded by actor.
    pub fn uploaded_by_actor(&self) -> &ActorRef {
        &self.uploaded_by_actor
    }
    /// Returns the aggregate uploaded at.
    pub fn uploaded_at(&self) -> DateTime<Utc> {
        self.uploaded_at
    }
    /// Returns the aggregate original file.
    pub fn original_file(&self) -> &document::OriginalFile {
        &self.original_file
    }
    /// Returns the aggregate storage ref.
    pub fn storage_ref(&self) -> &document::StorageRef {
        &self.storage_ref
    }
    /// Returns the aggregate virus scan status.
    pub fn virus_scan_status(&self) -> document::VirusScanStatus {
        self.virus_scan_status
    }
    /// Returns the aggregate pii redaction status.
    pub fn pii_redaction_status(&self) -> document::PiiRedactionStatus {
        self.pii_redaction_status
    }
    /// Returns the aggregate verification status.
    pub fn verification_status(&self) -> document::Status {
        self.verification_status
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }
    /// Returns the aggregate requires human review before use.
    pub fn requires_human_review_before_use(&self) -> bool {
        matches!(
            self.verification_status,
            document::Status::Received
                | document::Status::Extracting
                | document::Status::ExtractionFailed
                | document::Status::AwaitingReview
                | document::Status::QuarantinedRejected
        ) || !matches!(self.virus_scan_status, document::VirusScanStatus::Passed)
    }
}

#[derive(Debug, Clone, Default)]
/// Relationship-checked document builder used at this boundary.
pub struct DocumentBuilder {
    id: Option<DocumentId>,
    location_id: Option<LocationId>,
    subject: Option<DocumentSubject>,
    classification: Option<document::Classification>,
    source: Option<document::Source>,
    uploaded_by_actor: Option<ActorRef>,
    uploaded_at: Option<DateTime<Utc>>,
    original_file: Option<document::OriginalFile>,
    storage_ref: Option<document::StorageRef>,
    virus_scan_status: Option<document::VirusScanStatus>,
    pii_redaction_status: Option<document::PiiRedactionStatus>,
    verification_status: Option<document::Status>,
    audit_refs: Vec<crate::audit::EventId>,
}
impl DocumentBuilder {
    /// Returns the aggregate id.
    pub fn id(mut self, value: DocumentId) -> Self {
        self.id = Some(value);
        self
    }
    /// Returns the aggregate location id.
    pub fn location_id(mut self, value: LocationId) -> Self {
        self.location_id = Some(value);
        self
    }
    /// Returns the aggregate subject.
    pub fn subject(mut self, value: DocumentSubject) -> Self {
        self.subject = Some(value);
        self
    }
    /// Returns the aggregate classification.
    pub fn classification(mut self, value: document::Classification) -> Self {
        self.classification = Some(value);
        self
    }
    /// Returns the aggregate source.
    pub fn source(mut self, value: document::Source) -> Self {
        self.source = Some(value);
        self
    }
    /// Returns the aggregate uploaded by actor.
    pub fn uploaded_by_actor(mut self, value: ActorRef) -> Self {
        self.uploaded_by_actor = Some(value);
        self
    }
    /// Returns the aggregate uploaded at.
    pub fn uploaded_at(mut self, value: DateTime<Utc>) -> Self {
        self.uploaded_at = Some(value);
        self
    }
    /// Returns the aggregate original file.
    pub fn original_file(mut self, value: document::OriginalFile) -> Self {
        self.original_file = Some(value);
        self
    }
    /// Returns the aggregate storage ref.
    pub fn storage_ref(mut self, value: document::StorageRef) -> Self {
        self.storage_ref = Some(value);
        self
    }
    /// Returns the aggregate virus scan status.
    pub fn virus_scan_status(mut self, value: document::VirusScanStatus) -> Self {
        self.virus_scan_status = Some(value);
        self
    }
    /// Returns the aggregate pii redaction status.
    pub fn pii_redaction_status(mut self, value: document::PiiRedactionStatus) -> Self {
        self.pii_redaction_status = Some(value);
        self
    }
    /// Returns the aggregate verification status.
    pub fn verification_status(mut self, value: document::Status) -> Self {
        self.verification_status = Some(value);
        self
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(mut self, value: Vec<crate::audit::EventId>) -> Self {
        self.audit_refs = value;
        self
    }
    /// Validates the accumulated fields and builds the aggregate.
    pub fn build(self) -> std::result::Result<Document, DocumentError> {
        RawDocument {
            id: self.id.ok_or(DocumentError::IdRequired)?,
            location_id: self.location_id.ok_or(DocumentError::LocationIdRequired)?,
            subject: self.subject.ok_or(DocumentError::SubjectRequired)?,
            classification: self
                .classification
                .ok_or(DocumentError::ClassificationRequired)?,
            source: self.source.ok_or(DocumentError::SourceRequired)?,
            uploaded_by_actor: self
                .uploaded_by_actor
                .ok_or(DocumentError::UploadedByActorRequired)?,
            uploaded_at: self.uploaded_at.ok_or(DocumentError::UploadedAtRequired)?,
            original_file: self
                .original_file
                .ok_or(DocumentError::OriginalFileRequired)?,
            storage_ref: self.storage_ref.ok_or(DocumentError::StorageRefRequired)?,
            virus_scan_status: self
                .virus_scan_status
                .ok_or(DocumentError::VirusScanStatusRequired)?,
            pii_redaction_status: self
                .pii_redaction_status
                .ok_or(DocumentError::PiiRedactionStatusRequired)?,
            verification_status: self
                .verification_status
                .ok_or(DocumentError::VerificationStatusRequired)?,
            audit_refs: self.audit_refs,
        }
        .try_into_document()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject a document is evidence for.
pub enum DocumentSubject {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
}

/// Vaccine-record aggregate construction and rehydration failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VaccineRecordError {
    #[error("vaccine record id is required")]
    /// Represents the `IdRequired` semantic case.
    IdRequired,
    #[error("vaccine record pet id is required")]
    /// Represents the `PetIdRequired` semantic case.
    PetIdRequired,
    #[error("vaccine name is required")]
    /// Represents the `VaccineNameRequired` semantic case.
    VaccineNameRequired,
    #[error("vaccine source document id is required")]
    /// Represents the `SourceDocumentIdRequired` semantic case.
    SourceDocumentIdRequired,
    #[error("vaccine status is required")]
    /// Represents the `StatusRequired` semantic case.
    StatusRequired,
    #[error("vaccine effective date is required")]
    /// Represents the `EffectiveOnRequired` semantic case.
    EffectiveOnRequired,
    #[error("vaccine review gate is required")]
    /// Represents the `ReviewGateRequired` semantic case.
    ReviewGateRequired,
    #[error("vaccine expiration date must be after effective date")]
    /// Represents the `ExpirationMustBeAfterEffectiveDate` semantic case.
    ExpirationMustBeAfterEffectiveDate,
    #[error("expired vaccine status requires an expiration date")]
    /// Represents the `ExpiredStatusRequiresExpirationDate` semantic case.
    ExpiredStatusRequiresExpirationDate,
    #[error("vaccine exception status requires manager approval review gate")]
    /// Represents the `ExceptionStatusRequiresManagerApprovalReviewGate` semantic case.
    ExceptionStatusRequiresManagerApprovalReviewGate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Vaccine compliance record linking pet, vaccine name, expiration, proof document, and review status.
pub struct VaccineRecord {
    id: VaccineRecordId,
    pet_id: PetId,
    vaccine_name: policy::VaccineName,
    source_document_id: DocumentId,
    status: vaccine::Status,
    effective_on: NaiveDate,
    expires_on: Option<NaiveDate>,
    review_gate: policy::ReviewGate,
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawVaccineRecord {
    id: VaccineRecordId,
    pet_id: PetId,
    vaccine_name: policy::VaccineName,
    source_document_id: DocumentId,
    status: vaccine::Status,
    effective_on: NaiveDate,
    expires_on: Option<NaiveDate>,
    review_gate: policy::ReviewGate,
    #[serde(default)]
    audit_refs: Vec<crate::audit::EventId>,
}

impl RawVaccineRecord {
    fn try_into_record(self) -> std::result::Result<VaccineRecord, VaccineRecordError> {
        if self
            .expires_on
            .is_some_and(|expires_on| expires_on <= self.effective_on)
        {
            return Err(VaccineRecordError::ExpirationMustBeAfterEffectiveDate);
        }
        if matches!(self.status, vaccine::Status::VerifiedExpired) && self.expires_on.is_none() {
            return Err(VaccineRecordError::ExpiredStatusRequiresExpirationDate);
        }
        if matches!(
            self.status,
            vaccine::Status::ExceptionApproved | vaccine::Status::ExceptionRequested
        ) && self.review_gate != policy::ReviewGate::ManagerApproval
        {
            return Err(VaccineRecordError::ExceptionStatusRequiresManagerApprovalReviewGate);
        }
        Ok(VaccineRecord {
            id: self.id,
            pet_id: self.pet_id,
            vaccine_name: self.vaccine_name,
            source_document_id: self.source_document_id,
            status: self.status,
            effective_on: self.effective_on,
            expires_on: self.expires_on,
            review_gate: self.review_gate,
            audit_refs: self.audit_refs,
        })
    }
}

impl<'de> Deserialize<'de> for VaccineRecord {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawVaccineRecord::deserialize(deserializer)?
            .try_into_record()
            .map_err(serde::de::Error::custom)
    }
}

impl VaccineRecord {
    /// Starts checked construction of the aggregate.
    pub fn builder() -> VaccineRecordBuilder {
        VaccineRecordBuilder::default()
    }
    /// Returns the aggregate id.
    pub fn id(&self) -> VaccineRecordId {
        self.id
    }
    /// Returns the aggregate pet id.
    pub fn pet_id(&self) -> PetId {
        self.pet_id
    }
    /// Returns the aggregate vaccine name.
    pub fn vaccine_name(&self) -> &policy::VaccineName {
        &self.vaccine_name
    }
    /// Returns the aggregate source document id.
    pub fn source_document_id(&self) -> DocumentId {
        self.source_document_id
    }
    /// Returns the aggregate status.
    pub fn status(&self) -> vaccine::Status {
        self.status
    }
    /// Returns the aggregate effective on.
    pub fn effective_on(&self) -> NaiveDate {
        self.effective_on
    }
    /// Returns the aggregate expires on.
    pub fn expires_on(&self) -> Option<NaiveDate> {
        self.expires_on
    }
    /// Promotes the stored review gate into the semantic application value.
    pub fn review_gate(&self) -> policy::ReviewGate {
        self.review_gate.clone()
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }
    /// Reports whether vaccine proof is still unverified, rejected, or otherwise unsafe for compliance automation.
    pub fn requires_human_review_before_compliance(&self) -> bool {
        matches!(
            self.status,
            vaccine::Status::SuggestedExtracted
                | vaccine::Status::PendingReview
                | vaccine::Status::Rejected
                | vaccine::Status::ExceptionRequested
        )
    }
}

#[derive(Debug, Clone, Default)]
/// Relationship-checked vaccine record builder used at this boundary.
pub struct VaccineRecordBuilder {
    id: Option<VaccineRecordId>,
    pet_id: Option<PetId>,
    vaccine_name: Option<policy::VaccineName>,
    source_document_id: Option<DocumentId>,
    status: Option<vaccine::Status>,
    effective_on: Option<NaiveDate>,
    expires_on: Option<NaiveDate>,
    review_gate: Option<policy::ReviewGate>,
    audit_refs: Vec<crate::audit::EventId>,
}
impl VaccineRecordBuilder {
    /// Returns the aggregate id.
    pub fn id(mut self, value: VaccineRecordId) -> Self {
        self.id = Some(value);
        self
    }
    /// Returns the aggregate pet id.
    pub fn pet_id(mut self, value: PetId) -> Self {
        self.pet_id = Some(value);
        self
    }
    /// Returns the aggregate vaccine name.
    pub fn vaccine_name(mut self, value: policy::VaccineName) -> Self {
        self.vaccine_name = Some(value);
        self
    }
    /// Returns the aggregate source document id.
    pub fn source_document_id(mut self, value: DocumentId) -> Self {
        self.source_document_id = Some(value);
        self
    }
    /// Returns the aggregate status.
    pub fn status(mut self, value: vaccine::Status) -> Self {
        self.status = Some(value);
        self
    }
    /// Returns the aggregate effective on.
    pub fn effective_on(mut self, value: NaiveDate) -> Self {
        self.effective_on = Some(value);
        self
    }
    /// Returns the aggregate expires on.
    pub fn expires_on(mut self, value: NaiveDate) -> Self {
        self.expires_on = Some(value);
        self
    }
    /// Promotes the stored review gate into the semantic application value.
    pub fn review_gate(mut self, value: policy::ReviewGate) -> Self {
        self.review_gate = Some(value);
        self
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(mut self, value: Vec<crate::audit::EventId>) -> Self {
        self.audit_refs = value;
        self
    }
    /// Validates the accumulated fields and builds the aggregate.
    pub fn build(self) -> std::result::Result<VaccineRecord, VaccineRecordError> {
        RawVaccineRecord {
            id: self.id.ok_or(VaccineRecordError::IdRequired)?,
            pet_id: self.pet_id.ok_or(VaccineRecordError::PetIdRequired)?,
            vaccine_name: self
                .vaccine_name
                .ok_or(VaccineRecordError::VaccineNameRequired)?,
            source_document_id: self
                .source_document_id
                .ok_or(VaccineRecordError::SourceDocumentIdRequired)?,
            status: self.status.ok_or(VaccineRecordError::StatusRequired)?,
            effective_on: self
                .effective_on
                .ok_or(VaccineRecordError::EffectiveOnRequired)?,
            expires_on: self.expires_on,
            review_gate: self
                .review_gate
                .ok_or(VaccineRecordError::ReviewGateRequired)?,
            audit_refs: self.audit_refs,
        }
        .try_into_record()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Care note with author, visibility, subject, body, source, and review-sensitive timestamps.
pub struct CareNote {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    pub id: care_note::Id,
    /// Subject retained from source records for staff review, safety gates, and workflow joins.
    pub subject: care_note::Subject,
    /// Kind retained from source records for staff review, safety gates, and workflow joins.
    pub kind: care_note::Kind,
    /// Visibility retained from source records for staff review, safety gates, and workflow joins.
    pub visibility: care_note::Visibility,
    /// Body retained from source records for staff review, safety gates, and workflow joins.
    pub body: care_note::Body,
    /// Author retained from source records for staff review, safety gates, and workflow joins.
    pub author: ActorRef,
    /// Recorded at retained from source records for staff review, safety gates, and workflow joins.
    pub recorded_at: DateTime<Utc>,
    #[builder(default)]
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    pub audit_refs: Vec<crate::audit::EventId>,
}

impl CareNote {
    /// Serializable care-note history never bypasses review for customer-facing context.
    pub const fn is_customer_visible_without_review(&self) -> bool {
        false
    }
}

/// Checked evidence used to close an incident lifecycle.
pub mod incident_record {
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    use super::{ActorRef, IncidentError, IncidentId, approval, policy};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Serializable historical proof that an exact incident received manager approval for closure.
    pub struct ClosureEvidence {
        approval_id: approval::Id,
        incident_id: IncidentId,
        decided_by: ActorRef,
        decided_at: DateTime<Utc>,
    }

    /// Opaque one-use permission to close one exact incident.
    pub struct IncidentClosureAuthority {
        pub(super) evidence: ClosureEvidence,
    }

    impl ClosureEvidence {
        /// Promotes an exact approved incident decision into closure evidence.
        pub fn try_from_approval(
            approval: &approval::Record,
            incident_id: IncidentId,
        ) -> Result<Self, IncidentError> {
            if approval.target() != &approval::Target::Incident(incident_id)
                || approval.gate() != &policy::ReviewGate::ManagerApproval
            {
                return Err(IncidentError::ClosureApprovalMismatch);
            }
            let approval::Lifecycle::Approved {
                decided_by,
                decided_at,
            } = approval.lifecycle()
            else {
                return Err(IncidentError::ClosureApprovalMismatch);
            };
            Ok(Self {
                approval_id: approval.id(),
                incident_id,
                decided_by: decided_by.clone(),
                decided_at: *decided_at,
            })
        }

        /// Exact incident target approved for closure.
        pub const fn incident_id(&self) -> IncidentId {
            self.incident_id
        }

        /// Reviewer identity recorded by the accepted historical decision.
        pub const fn decided_by(&self) -> &ActorRef {
            &self.decided_by
        }
    }

    /// Test-only issuer for the incident closure capability protocol.
    ///
    /// Production issuance is deliberately absent until a real authenticated reviewer boundary
    /// owns current actor, role, and scope validation.
    #[cfg(test)]
    pub(crate) fn issue_incident_closure_authority(
        approval: &approval::Record,
        incident_id: IncidentId,
        current_reviewer: &ActorRef,
    ) -> Result<IncidentClosureAuthority, IncidentError> {
        let evidence = ClosureEvidence::try_from_approval(approval, incident_id)?;
        if evidence.decided_by() != current_reviewer {
            return Err(IncidentError::ClosureApprovalMismatch);
        }
        Ok(IncidentClosureAuthority { evidence })
    }

    #[cfg(test)]
    mod tests {
        use chrono::{TimeZone, Utc};
        use uuid::Uuid;

        use super::{super::Incident, *};
        use crate::{entities, incident, policy};

        #[test]
        fn current_reviewer_authority_issues_one_incident_bound_closure_capability() {
            let incident_id = IncidentId::new(uuid::Uuid::from_u128(1));
            let reviewer = ActorRef::Manager {
                manager_id: entities::ManagerId::try_new("manager-1").unwrap(),
            };
            let decided_at = Utc.with_ymd_and_hms(2026, 8, 15, 1, 0, 0).unwrap();
            let approval = approval::Record::builder()
                .id(approval::Id::new(Uuid::from_u128(2)))
                .target(approval::Target::Incident(incident_id))
                .gate(policy::ReviewGate::ManagerApproval)
                .lifecycle(approval::Lifecycle::Approved {
                    decided_by: reviewer.clone(),
                    decided_at,
                })
                .requested_by(ActorRef::System)
                .requested_at(decided_at)
                .build()
                .unwrap();
            let incident = Incident::builder()
                .id(incident_id)
                .location_id(entities::LocationId::new(Uuid::from_u128(3)))
                .primary_subject(entities::IncidentSubject::Pet(entities::PetId::new(
                    Uuid::from_u128(4),
                )))
                .category(incident::Category::Medication)
                .severity(incident::Severity::High)
                .status(incident::Status::NeedsManagerReview)
                .reported_by(ActorRef::System)
                .reported_at(decided_at)
                .summary(incident::Summary::try_new("missed dose").unwrap())
                .required_review_gates(vec![policy::ReviewGate::ManagerApproval])
                .build()
                .unwrap();

            let authority =
                issue_incident_closure_authority(&approval, incident_id, &reviewer).unwrap();
            let closed = incident.close_with(authority).unwrap();

            assert_eq!(closed.status(), incident::Status::Closed);
            assert!(closed.closure_evidence().is_some());
        }
    }
}

/// Incident aggregate construction and rehydration failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IncidentError {
    #[error("incident id is required")]
    /// Represents the `IdRequired` semantic case.
    IdRequired,
    #[error("incident location id is required")]
    /// Represents the `LocationIdRequired` semantic case.
    LocationIdRequired,
    #[error("incident primary subject is required")]
    /// Represents the `PrimarySubjectRequired` semantic case.
    PrimarySubjectRequired,
    #[error("incident category is required")]
    /// Represents the `CategoryRequired` semantic case.
    CategoryRequired,
    #[error("incident severity is required")]
    /// Represents the `SeverityRequired` semantic case.
    SeverityRequired,
    #[error("incident status is required")]
    /// Represents the `StatusRequired` semantic case.
    StatusRequired,
    #[error("incident reporter is required")]
    /// Represents the `ReportedByRequired` semantic case.
    ReportedByRequired,
    #[error("incident report time is required")]
    /// Represents the `ReportedAtRequired` semantic case.
    ReportedAtRequired,
    #[error("incident summary is required")]
    /// Represents the `SummaryRequired` semantic case.
    SummaryRequired,
    #[error("incident requires manager approval review gate")]
    /// Represents the `IncidentRequiresManagerApprovalReviewGate` semantic case.
    IncidentRequiresManagerApprovalReviewGate,
    #[error("customer-message incident requires customer message approval review gate")]
    /// Represents the `CustomerMessageIncidentRequiresCustomerMessageApprovalReviewGate` semantic case.
    CustomerMessageIncidentRequiresCustomerMessageApprovalReviewGate,
    #[error("closed incident requires exact manager closure evidence")]
    /// A gate label or observed closed status cannot substitute for exact closure proof.
    ClosedRequiresClosureEvidence,
    #[error("incident closure evidence is bound to a different target or decision")]
    /// Closure approval was not an approved manager decision for this incident.
    ClosureApprovalMismatch,
    #[error("only a closed incident may carry closure evidence")]
    /// Historical closure evidence cannot be attached to an active incident state.
    ClosureEvidenceRequiresClosedStatus,
    #[error("closed incident rehydration requires a trusted approval aggregate")]
    /// Generic serde cannot promote historical closure fields into a closed lifecycle.
    ClosedRehydrationRequiresTrustedApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Incident record used for manager attention, safety follow-up, customer messaging, and audit evidence.
pub struct Incident {
    id: IncidentId,
    location_id: LocationId,
    primary_subject: IncidentSubject,
    category: incident::Category,
    severity: incident::Severity,
    status: incident::Status,
    reported_by: ActorRef,
    reported_at: DateTime<Utc>,
    summary: incident::Summary,
    required_review_gates: Vec<policy::ReviewGate>,
    closure_evidence: Option<incident_record::ClosureEvidence>,
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawIncident {
    id: IncidentId,
    location_id: LocationId,
    primary_subject: IncidentSubject,
    category: incident::Category,
    severity: incident::Severity,
    status: incident::Status,
    reported_by: ActorRef,
    reported_at: DateTime<Utc>,
    summary: incident::Summary,
    #[serde(default)]
    required_review_gates: Vec<policy::ReviewGate>,
    #[serde(default)]
    closure_evidence: Option<serde::de::IgnoredAny>,
    #[serde(default)]
    audit_refs: Vec<crate::audit::EventId>,
}

impl RawIncident {
    fn try_into_incident(self) -> std::result::Result<Incident, IncidentError> {
        if matches!(
            self.severity,
            incident::Severity::High | incident::Severity::Critical
        ) && !self
            .required_review_gates
            .contains(&policy::ReviewGate::ManagerApproval)
        {
            return Err(IncidentError::IncidentRequiresManagerApprovalReviewGate);
        }
        if matches!(self.status, incident::Status::CustomerMessageReview)
            && !self
                .required_review_gates
                .contains(&policy::ReviewGate::CustomerMessageApproval)
        {
            return Err(
                IncidentError::CustomerMessageIncidentRequiresCustomerMessageApprovalReviewGate,
            );
        }
        if matches!(self.status, incident::Status::Closed) {
            return Err(IncidentError::ClosedRehydrationRequiresTrustedApproval);
        } else if self.closure_evidence.is_some() {
            return Err(IncidentError::ClosureEvidenceRequiresClosedStatus);
        }
        Ok(Incident {
            id: self.id,
            location_id: self.location_id,
            primary_subject: self.primary_subject,
            category: self.category,
            severity: self.severity,
            status: self.status,
            reported_by: self.reported_by,
            reported_at: self.reported_at,
            summary: self.summary,
            required_review_gates: self.required_review_gates,
            closure_evidence: None,
            audit_refs: self.audit_refs,
        })
    }
}

impl<'de> Deserialize<'de> for Incident {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawIncident::deserialize(deserializer)?
            .try_into_incident()
            .map_err(serde::de::Error::custom)
    }
}

impl Incident {
    /// Starts checked construction of the aggregate.
    pub fn builder() -> IncidentBuilder {
        IncidentBuilder::default()
    }
    /// Closes this incident only by consuming opaque current authority for this exact target.
    pub fn close_with(
        mut self,
        authority: incident_record::IncidentClosureAuthority,
    ) -> std::result::Result<Self, IncidentError> {
        let evidence = authority.evidence;
        if evidence.incident_id() != self.id {
            return Err(IncidentError::ClosureApprovalMismatch);
        }
        self.status = incident::Status::Closed;
        self.closure_evidence = Some(evidence);
        Ok(self)
    }

    /// Returns persisted historical closure evidence, when the incident is closed.
    pub const fn closure_evidence(&self) -> Option<&incident_record::ClosureEvidence> {
        self.closure_evidence.as_ref()
    }

    /// Returns the aggregate id.
    pub fn id(&self) -> IncidentId {
        self.id
    }
    /// Returns the aggregate location id.
    pub fn location_id(&self) -> LocationId {
        self.location_id
    }
    /// Returns the aggregate primary subject.
    pub fn primary_subject(&self) -> &IncidentSubject {
        &self.primary_subject
    }
    /// Returns the aggregate category.
    pub fn category(&self) -> incident::Category {
        self.category
    }
    /// Returns the aggregate severity.
    pub fn severity(&self) -> incident::Severity {
        self.severity
    }
    /// Returns the aggregate status.
    pub fn status(&self) -> incident::Status {
        self.status
    }
    /// Returns the aggregate reported by.
    pub fn reported_by(&self) -> &ActorRef {
        &self.reported_by
    }
    /// Returns the aggregate reported at.
    pub fn reported_at(&self) -> DateTime<Utc> {
        self.reported_at
    }
    /// Returns the aggregate summary.
    pub fn summary(&self) -> &incident::Summary {
        &self.summary
    }
    /// Returns the aggregate required review gates.
    pub fn required_review_gates(&self) -> &[policy::ReviewGate] {
        &self.required_review_gates
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }
    /// Reports whether the incident is still active enough to require manager attention.
    pub fn requires_manager_attention(&self) -> bool {
        matches!(
            self.status,
            incident::Status::NeedsManagerReview | incident::Status::LegalHold
        ) || matches!(
            self.severity,
            incident::Severity::High | incident::Severity::Critical
        ) || self
            .required_review_gates
            .contains(&policy::ReviewGate::ManagerApproval)
    }
}

#[derive(Debug, Clone, Default)]
/// Relationship-checked incident builder used at this boundary.
pub struct IncidentBuilder {
    id: Option<IncidentId>,
    location_id: Option<LocationId>,
    primary_subject: Option<IncidentSubject>,
    category: Option<incident::Category>,
    severity: Option<incident::Severity>,
    status: Option<incident::Status>,
    reported_by: Option<ActorRef>,
    reported_at: Option<DateTime<Utc>>,
    summary: Option<incident::Summary>,
    required_review_gates: Vec<policy::ReviewGate>,
    audit_refs: Vec<crate::audit::EventId>,
}
impl IncidentBuilder {
    /// Returns the aggregate id.
    pub fn id(mut self, value: IncidentId) -> Self {
        self.id = Some(value);
        self
    }
    /// Returns the aggregate location id.
    pub fn location_id(mut self, value: LocationId) -> Self {
        self.location_id = Some(value);
        self
    }
    /// Returns the aggregate primary subject.
    pub fn primary_subject(mut self, value: IncidentSubject) -> Self {
        self.primary_subject = Some(value);
        self
    }
    /// Returns the aggregate category.
    pub fn category(mut self, value: incident::Category) -> Self {
        self.category = Some(value);
        self
    }
    /// Returns the aggregate severity.
    pub fn severity(mut self, value: incident::Severity) -> Self {
        self.severity = Some(value);
        self
    }
    /// Returns the aggregate status.
    pub fn status(mut self, value: incident::Status) -> Self {
        self.status = Some(value);
        self
    }
    /// Returns the aggregate reported by.
    pub fn reported_by(mut self, value: ActorRef) -> Self {
        self.reported_by = Some(value);
        self
    }
    /// Returns the aggregate reported at.
    pub fn reported_at(mut self, value: DateTime<Utc>) -> Self {
        self.reported_at = Some(value);
        self
    }
    /// Returns the aggregate summary.
    pub fn summary(mut self, value: incident::Summary) -> Self {
        self.summary = Some(value);
        self
    }
    /// Returns the aggregate required review gates.
    pub fn required_review_gates(mut self, value: Vec<policy::ReviewGate>) -> Self {
        self.required_review_gates = value;
        self
    }
    /// Returns the aggregate audit refs.
    pub fn audit_refs(mut self, value: Vec<crate::audit::EventId>) -> Self {
        self.audit_refs = value;
        self
    }
    /// Validates the accumulated fields and builds the aggregate.
    pub fn build(self) -> std::result::Result<Incident, IncidentError> {
        RawIncident {
            id: self.id.ok_or(IncidentError::IdRequired)?,
            location_id: self.location_id.ok_or(IncidentError::LocationIdRequired)?,
            primary_subject: self
                .primary_subject
                .ok_or(IncidentError::PrimarySubjectRequired)?,
            category: self.category.ok_or(IncidentError::CategoryRequired)?,
            severity: self.severity.ok_or(IncidentError::SeverityRequired)?,
            status: self.status.ok_or(IncidentError::StatusRequired)?,
            reported_by: self.reported_by.ok_or(IncidentError::ReportedByRequired)?,
            reported_at: self.reported_at.ok_or(IncidentError::ReportedAtRequired)?,
            summary: self.summary.ok_or(IncidentError::SummaryRequired)?,
            required_review_gates: self.required_review_gates,
            closure_evidence: None,
            audit_refs: self.audit_refs,
        }
        .try_into_incident()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject affected by an incident.
pub enum IncidentSubject {
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Resort location record participating in the workflow.
    Location(LocationId),
}

/// Message aggregate vocabulary and checked lifecycle evidence.
pub mod message_record {
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    use super::{ActorRef, MessageId, approval, message, policy};

    /// Message aggregate construction and lifecycle-promotion failures.
    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        #[error("message id is required")]
        /// Represents the `IdRequired` semantic case.
        IdRequired,
        #[error("message subject is required")]
        /// Represents the `SubjectRequired` semantic case.
        SubjectRequired,
        #[error("message direction is required")]
        /// Represents the `DirectionRequired` semantic case.
        DirectionRequired,
        #[error("message channel is required")]
        /// Represents the `ChannelRequired` semantic case.
        ChannelRequired,
        #[error("message status is required")]
        /// Represents the `StatusRequired` semantic case.
        StatusRequired,
        #[error("message body reference is required")]
        /// Represents the `BodyRefRequired` semantic case.
        BodyRefRequired,
        #[error("outbound draft cannot carry queued, attempted, or delivered status")]
        /// Represents the `DraftCannotCarryDeliveryStatus` semantic case.
        DraftCannotCarryDeliveryStatus,
        #[error("queued or approved outbound message requires approval decision evidence")]
        /// Represents the `QueuedOrApprovedOutboundRequiresApprovalEvidence` semantic case.
        QueuedOrApprovedOutboundRequiresApprovalEvidence,
        #[error("inbound messages cannot carry outbound delivery lifecycle status")]
        /// Represents the `InboundCannotCarryOutboundDeliveryStatus` semantic case.
        InboundCannotCarryOutboundDeliveryStatus,
        #[error("outbound sent message requires attempted, delivered, or failed status")]
        /// Represents the `SentRequiresAttemptedDeliveredOrFailedStatus` semantic case.
        SentRequiresAttemptedDeliveredOrFailedStatus,
        #[error(
            "message approval evidence must be an approved decision for the requested message and gate"
        )]
        /// Represents the `ApprovalEvidenceMismatch` semantic case.
        ApprovalEvidenceMismatch,
        #[error("trusted reviewer authority does not match the approval decision")]
        /// Opaque reviewer authority was issued for another actor, target, gate, or approval.
        ReviewerAuthorityMismatch,
        #[error("message queue capability does not satisfy the draft approval gate")]
        /// Represents the `QueueCapabilityGateMismatch` semantic case.
        QueueCapabilityGateMismatch,
        #[error("message queue capability is bound to a different message")]
        /// Represents the `QueueCapabilityTargetMismatch` semantic case.
        QueueCapabilityTargetMismatch,
        #[error("only approval-requested outbound drafts can be queued")]
        /// Represents the `OnlyApprovalRequestedDraftsCanBeQueued` semantic case.
        OnlyApprovalRequestedDraftsCanBeQueued,
        #[error("queued or sent message rehydration requires opaque queue authorization")]
        /// Generic serde cannot promote historical approval fields into executable lifecycle state.
        ExecutableRehydrationRequiresQueueAuthorization,
    }

    /// Result alias for message aggregate construction and lifecycle promotion.
    pub type Result<T> = std::result::Result<T, Error>;

    /// Serializable historical approval evidence for a message lifecycle.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct ApprovalEvidence {
        /// Approval decision id when the evidence came from an owned approval record.
        approval_id: Option<approval::Id>,
        /// Exact message target approved by the owned decision.
        message_id: MessageId,
        /// Review gate that approved or historically guarded this message lifecycle.
        gate: policy::ReviewGate,
        /// Actor that approved queueing when known from the approval record.
        decided_by: Option<ActorRef>,
        /// Decision time when known from the approval record.
        decided_at: Option<DateTime<Utc>>,
    }

    impl ApprovalEvidence {
        /// Promotes an approved review record into serializable message approval evidence.
        pub fn try_from_approval(
            approval: &approval::Record,
            message_id: MessageId,
            gate: policy::ReviewGate,
        ) -> Result<Self> {
            if approval.target() != &approval::Target::Message(message_id)
                || approval.gate() != &gate
            {
                return Err(Error::ApprovalEvidenceMismatch);
            }
            let approval::Lifecycle::Approved {
                decided_by,
                decided_at,
            } = approval.lifecycle()
            else {
                return Err(Error::ApprovalEvidenceMismatch);
            };
            Ok(Self {
                approval_id: Some(approval.id()),
                message_id,
                gate,
                decided_by: Some(decided_by.clone()),
                decided_at: Some(*decided_at),
            })
        }

        pub(super) const fn is_owned_decision(&self) -> bool {
            self.approval_id.is_some() && self.decided_by.is_some() && self.decided_at.is_some()
        }

        /// Approval record id retained as historical evidence, when available.
        pub const fn approval_id(&self) -> Option<approval::Id> {
            self.approval_id
        }

        /// Exact message target retained from the approval decision.
        pub const fn message_id(&self) -> MessageId {
            self.message_id
        }

        /// Review gate retained as historical evidence.
        pub fn gate(&self) -> &policy::ReviewGate {
            &self.gate
        }
    }

    /// Opaque, non-serializable proof that the trusted application authorization boundary
    /// admitted one reviewer for one approval, message target, and review gate.
    #[derive(Debug)]
    pub struct ReviewerAuthority {
        approval_id: approval::Id,
        message_id: MessageId,
        gate: policy::ReviewGate,
        reviewer: ActorRef,
    }

    /// Test-only issuer for the capability protocol.
    ///
    /// Production issuance is intentionally absent in the pre-data phase. A future authenticated
    /// actor adapter must own that root of trust; raw actor labels and persisted approvals are not
    /// substitutes. Until then the executable message-queue path remains fail-closed.
    #[cfg(test)]
    pub(crate) fn issue_reviewer_authority(
        approval_id: approval::Id,
        message_id: MessageId,
        gate: policy::ReviewGate,
        reviewer: ActorRef,
    ) -> ReviewerAuthority {
        ReviewerAuthority {
            approval_id,
            message_id,
            gate,
            reviewer,
        }
    }

    /// Opaque, target-bound, non-serializable authority to admit one approved message to a queue.
    #[derive(Debug)]
    pub struct QueueAuthorization {
        pub(super) message_id: MessageId,
        pub(super) evidence: ApprovalEvidence,
    }

    /// Combines historical approval evidence with current, opaque reviewer authority.
    pub fn authorize_queue(
        approval: &approval::Record,
        reviewer_authority: ReviewerAuthority,
    ) -> Result<(ApprovalEvidence, QueueAuthorization)> {
        let message_id = reviewer_authority.message_id;
        let gate = reviewer_authority.gate;
        let approval::Lifecycle::Approved { decided_by, .. } = approval.lifecycle() else {
            return Err(Error::ReviewerAuthorityMismatch);
        };
        if approval.id() != reviewer_authority.approval_id
            || approval.target() != &approval::Target::Message(message_id)
            || approval.gate() != &gate
            || decided_by != &reviewer_authority.reviewer
        {
            return Err(Error::ReviewerAuthorityMismatch);
        }
        let evidence = ApprovalEvidence::try_from_approval(approval, message_id, gate)?;
        let authorization = QueueAuthorization {
            message_id,
            evidence: evidence.clone(),
        };
        Ok((evidence, authorization))
    }

    #[cfg(test)]
    mod tests {
        use chrono::{TimeZone, Utc};
        use uuid::Uuid;

        use super::*;

        #[test]
        fn matching_trusted_reviewer_authority_issues_one_target_bound_queue_capability() {
            let message_id = MessageId::new(Uuid::from_u128(500));
            let approval_id = approval::Id::new(Uuid::from_u128(501));
            let reviewer = ActorRef::Manager {
                manager_id: super::super::ManagerId::try_new("trusted-reviewer").unwrap(),
            };
            let approval = approval::Record::builder()
                .id(approval_id)
                .target(approval::Target::Message(message_id))
                .gate(policy::ReviewGate::CustomerMessageApproval)
                .lifecycle(approval::Lifecycle::Approved {
                    decided_by: reviewer.clone(),
                    decided_at: Utc.with_ymd_and_hms(2026, 8, 15, 10, 0, 0).unwrap(),
                })
                .requested_by(ActorRef::System)
                .requested_at(Utc.with_ymd_and_hms(2026, 8, 15, 9, 0, 0).unwrap())
                .build()
                .unwrap();
            let authority = issue_reviewer_authority(
                approval_id,
                message_id,
                policy::ReviewGate::CustomerMessageApproval,
                reviewer,
            );

            let (evidence, authorization) = authorize_queue(&approval, authority).unwrap();
            assert_eq!(evidence.message_id(), message_id);
            assert_eq!(authorization.message_id, message_id);
        }
    }

    /// Checked message lifecycle; variants carry exactly the evidence legal for their phase.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Lifecycle {
        /// Inbound source message with no outbound approval or queue authority.
        InboundReceived,
        /// Outbound draft or review-requested draft that has not entered the queue.
        OutboundDraft {
            /// Draft-side status, limited to statuses that cannot imply send authority.
            status: message::Status,
            /// Review gate requested for this draft, if any.
            approval_gate: Option<policy::ReviewGate>,
        },
        /// Outbound message admitted to the queue by approval evidence and queue capability.
        OutboundQueued {
            /// Queue-side status before provider delivery evidence exists.
            status: message::Status,
            /// Historical approval evidence that justified queue admission.
            approval_evidence: ApprovalEvidence,
        },
        /// Outbound delivery path with approval evidence and attempt/result status.
        OutboundSent {
            /// Attempt/result status.
            status: message::Status,
            /// Historical approval evidence that justified the outbound send path.
            approval_evidence: ApprovalEvidence,
        },
    }

    impl Lifecycle {
        pub(super) fn try_new(
            direction: message::Direction,
            status: message::Status,
            approval_gate: Option<policy::ReviewGate>,
        ) -> Result<Self> {
            match direction {
                message::Direction::InboundReceived => {
                    if matches!(
                        status,
                        message::Status::ApprovedToQueue
                            | message::Status::Queued
                            | message::Status::SendAttempted
                            | message::Status::Delivered
                    ) {
                        return Err(Error::InboundCannotCarryOutboundDeliveryStatus);
                    }
                    Ok(Self::InboundReceived)
                }
                message::Direction::OutboundDraft => {
                    if !matches!(
                        status,
                        message::Status::DraftCreated
                            | message::Status::ApprovalRequested
                            | message::Status::Suppressed
                            | message::Status::Cancelled
                    ) {
                        return Err(Error::DraftCannotCarryDeliveryStatus);
                    }
                    Ok(Self::OutboundDraft {
                        status,
                        approval_gate,
                    })
                }
                message::Direction::OutboundQueued | message::Direction::OutboundSent => {
                    Err(Error::QueuedOrApprovedOutboundRequiresApprovalEvidence)
                }
            }
        }

        pub(super) fn try_from_persisted(
            message_id: MessageId,
            direction: message::Direction,
            status: message::Status,
            approval_gate: Option<policy::ReviewGate>,
            approval_evidence: Option<ApprovalEvidence>,
        ) -> Result<Self> {
            match direction {
                message::Direction::InboundReceived | message::Direction::OutboundDraft => {
                    if approval_evidence.is_some() {
                        return Err(Error::ApprovalEvidenceMismatch);
                    }
                    Self::try_new(direction, status, approval_gate)
                }
                message::Direction::OutboundQueued => {
                    let evidence = approval_evidence
                        .filter(ApprovalEvidence::is_owned_decision)
                        .ok_or(Error::QueuedOrApprovedOutboundRequiresApprovalEvidence)?;
                    if evidence.message_id() != message_id
                        || approval_gate.as_ref() != Some(evidence.gate())
                    {
                        return Err(Error::ApprovalEvidenceMismatch);
                    }
                    if !matches!(
                        status,
                        message::Status::ApprovedToQueue | message::Status::Queued
                    ) {
                        return Err(Error::DraftCannotCarryDeliveryStatus);
                    }
                    Ok(Self::OutboundQueued {
                        status,
                        approval_evidence: evidence,
                    })
                }
                message::Direction::OutboundSent => {
                    let evidence = approval_evidence
                        .filter(ApprovalEvidence::is_owned_decision)
                        .ok_or(Error::QueuedOrApprovedOutboundRequiresApprovalEvidence)?;
                    if evidence.message_id() != message_id
                        || approval_gate.as_ref() != Some(evidence.gate())
                    {
                        return Err(Error::ApprovalEvidenceMismatch);
                    }
                    if !matches!(
                        status,
                        message::Status::SendAttempted
                            | message::Status::Delivered
                            | message::Status::Failed
                    ) {
                        return Err(Error::SentRequiresAttemptedDeliveredOrFailedStatus);
                    }
                    Ok(Self::OutboundSent {
                        status,
                        approval_evidence: evidence,
                    })
                }
            }
        }

        pub(super) fn direction(&self) -> message::Direction {
            match self {
                Self::InboundReceived => message::Direction::InboundReceived,
                Self::OutboundDraft { .. } => message::Direction::OutboundDraft,
                Self::OutboundQueued { .. } => message::Direction::OutboundQueued,
                Self::OutboundSent { .. } => message::Direction::OutboundSent,
            }
        }

        pub(super) fn status(&self) -> message::Status {
            match self {
                Self::InboundReceived => message::Status::DraftCreated,
                Self::OutboundDraft { status, .. }
                | Self::OutboundQueued { status, .. }
                | Self::OutboundSent { status, .. } => *status,
            }
        }

        pub(super) fn approval_gate(&self) -> Option<policy::ReviewGate> {
            match self {
                Self::InboundReceived => None,
                Self::OutboundDraft { approval_gate, .. } => approval_gate.clone(),
                Self::OutboundQueued {
                    approval_evidence, ..
                }
                | Self::OutboundSent {
                    approval_evidence, ..
                } => Some(approval_evidence.gate.clone()),
            }
        }

        pub(super) fn approval_evidence(&self) -> Option<ApprovalEvidence> {
            match self {
                Self::OutboundQueued {
                    approval_evidence, ..
                }
                | Self::OutboundSent {
                    approval_evidence, ..
                } => Some(approval_evidence.clone()),
                Self::InboundReceived | Self::OutboundDraft { .. } => None,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Customer/internal message record that tracks subject, channel, draft/reference body, approval, and delivery state.
pub struct Message {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    id: MessageId,
    /// Subject retained from source records for staff review, safety gates, and workflow joins.
    subject: MessageSubject,
    /// Channel retained from source records for staff review, safety gates, and workflow joins.
    channel: message::Channel,
    /// Body ref retained from source records for staff review, safety gates, and workflow joins.
    body_ref: message::BodyRef,
    /// Checked lifecycle evidence that replaces independent direction/status/gate products.
    lifecycle: message_record::Lifecycle,
    /// Audit refs retained from source records for staff review, safety gates, and workflow joins.
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Deserialize)]
struct RawMessage {
    id: MessageId,
    subject: MessageSubject,
    direction: message::Direction,
    channel: message::Channel,
    status: message::Status,
    body_ref: message::BodyRef,
    approval_gate: Option<policy::ReviewGate>,
    #[serde(default)]
    approval_evidence: Option<serde::de::IgnoredAny>,
    #[serde(default)]
    audit_refs: Vec<crate::audit::EventId>,
}

#[derive(Serialize)]
struct SerializedMessage<'a> {
    id: MessageId,
    subject: &'a MessageSubject,
    direction: message::Direction,
    channel: message::Channel,
    status: message::Status,
    body_ref: &'a message::BodyRef,
    approval_gate: Option<policy::ReviewGate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_evidence: Option<&'a message_record::ApprovalEvidence>,
    audit_refs: &'a [crate::audit::EventId],
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let approval_evidence = self.lifecycle.approval_evidence();
        SerializedMessage {
            id: self.id,
            subject: &self.subject,
            direction: self.direction(),
            channel: self.channel,
            status: self.status(),
            body_ref: &self.body_ref,
            approval_gate: self.approval_gate(),
            approval_evidence: approval_evidence.as_ref(),
            audit_refs: &self.audit_refs,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawMessage::deserialize(deserializer)?;
        Self::try_from_persisted(raw).map_err(serde::de::Error::custom)
    }
}

impl Message {
    fn try_from_persisted(raw: RawMessage) -> message_record::Result<Self> {
        if raw.approval_evidence.is_some()
            || matches!(
                raw.status,
                message::Status::ApprovedToQueue
                    | message::Status::Queued
                    | message::Status::SendAttempted
                    | message::Status::Delivered
            )
        {
            return Err(message_record::Error::ExecutableRehydrationRequiresQueueAuthorization);
        }
        let lifecycle = message_record::Lifecycle::try_from_persisted(
            raw.id,
            raw.direction,
            raw.status,
            raw.approval_gate,
            None,
        )?;
        Ok(Self {
            id: raw.id,
            subject: raw.subject,
            channel: raw.channel,
            body_ref: raw.body_ref,
            lifecycle,
            audit_refs: raw.audit_refs,
        })
    }

    /// Starts a checked message aggregate builder for compatibility with existing call sites.
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    /// Constructs an approval-requested outbound draft without exposing an invalid lifecycle product.
    pub fn approval_requested_outbound_draft(
        id: MessageId,
        subject: MessageSubject,
        channel: message::Channel,
        body_ref: message::BodyRef,
        approval_gate: policy::ReviewGate,
    ) -> Self {
        Self {
            id,
            subject,
            channel,
            body_ref,
            lifecycle: message_record::Lifecycle::OutboundDraft {
                status: message::Status::ApprovalRequested,
                approval_gate: Some(approval_gate),
            },
            audit_refs: Vec::new(),
        }
    }

    /// Promotes an approved draft into the queue lifecycle using opaque executable authority.
    pub fn queue_with(
        mut self,
        authorization: message_record::QueueAuthorization,
    ) -> message_record::Result<Self> {
        let message_record::Lifecycle::OutboundDraft {
            status: message::Status::ApprovalRequested,
            approval_gate: Some(required_gate),
        } = self.lifecycle
        else {
            return Err(message_record::Error::OnlyApprovalRequestedDraftsCanBeQueued);
        };
        if authorization.message_id != self.id {
            return Err(message_record::Error::QueueCapabilityTargetMismatch);
        }
        if authorization.evidence.gate() != &required_gate {
            return Err(message_record::Error::QueueCapabilityGateMismatch);
        }
        self.lifecycle = message_record::Lifecycle::OutboundQueued {
            status: message::Status::Queued,
            approval_evidence: authorization.evidence,
        };
        Ok(self)
    }

    /// Message identifier used by workflow, storage, and review joins.
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// Subject this message refers to.
    pub fn subject(&self) -> &MessageSubject {
        &self.subject
    }

    /// Direction derived from the checked lifecycle variant.
    pub fn direction(&self) -> message::Direction {
        self.lifecycle.direction()
    }

    /// Delivery/review status derived from the checked lifecycle variant.
    pub fn status(&self) -> message::Status {
        self.lifecycle.status()
    }

    /// Channel selected for this message.
    pub fn channel(&self) -> message::Channel {
        self.channel
    }

    /// Body reference containing draft/source evidence.
    pub fn body_ref(&self) -> &message::BodyRef {
        &self.body_ref
    }

    /// Serializable review gate evidence attached to lifecycle variants that require it.
    pub fn approval_gate(&self) -> Option<policy::ReviewGate> {
        self.lifecycle.approval_gate()
    }

    /// Audit refs attached to this message.
    pub fn audit_refs(&self) -> &[crate::audit::EventId] {
        &self.audit_refs
    }

    /// Reports whether the message is still a draft or awaiting approval before any outbound send.
    pub fn requires_approval_before_send(&self) -> bool {
        self.approval_gate().is_some()
            || matches!(self.status(), message::Status::ApprovalRequested)
            || matches!(self.direction(), message::Direction::OutboundDraft)
    }
}

/// Builder for checked message aggregates.
#[derive(Debug, Clone, Default)]
pub struct MessageBuilder {
    id: Option<MessageId>,
    subject: Option<MessageSubject>,
    direction: Option<message::Direction>,
    channel: Option<message::Channel>,
    status: Option<message::Status>,
    body_ref: Option<message::BodyRef>,
    approval_gate: Option<policy::ReviewGate>,
    audit_refs: Vec<crate::audit::EventId>,
}

impl MessageBuilder {
    /// Sets the message id.
    pub fn id(mut self, id: MessageId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the message subject.
    pub fn subject(mut self, subject: MessageSubject) -> Self {
        self.subject = Some(subject);
        self
    }

    /// Sets the message direction.
    pub fn direction(mut self, direction: message::Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Sets the message channel.
    pub fn channel(mut self, channel: message::Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    /// Sets the lifecycle status.
    pub fn status(mut self, status: message::Status) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the body reference.
    pub fn body_ref(mut self, body_ref: message::BodyRef) -> Self {
        self.body_ref = Some(body_ref);
        self
    }

    /// Sets the approval gate evidence.
    pub fn approval_gate(mut self, approval_gate: policy::ReviewGate) -> Self {
        self.approval_gate = Some(approval_gate);
        self
    }

    /// Replaces audit refs.
    pub fn audit_refs(mut self, audit_refs: Vec<crate::audit::EventId>) -> Self {
        self.audit_refs = audit_refs;
        self
    }

    /// Builds a message only after lifecycle/evidence invariants pass.
    pub fn build(self) -> message_record::Result<Message> {
        Message::try_from_persisted(RawMessage {
            id: self.id.ok_or(message_record::Error::IdRequired)?,
            subject: self.subject.ok_or(message_record::Error::SubjectRequired)?,
            direction: self
                .direction
                .ok_or(message_record::Error::DirectionRequired)?,
            channel: self.channel.ok_or(message_record::Error::ChannelRequired)?,
            status: self.status.ok_or(message_record::Error::StatusRequired)?,
            body_ref: self
                .body_ref
                .ok_or(message_record::Error::BodyRefRequired)?,
            approval_gate: self.approval_gate,
            approval_evidence: None,
            audit_refs: self.audit_refs,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Entity or workflow subject that a message refers to.
pub enum MessageSubject {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Pet record participating in the workflow.
    Pet(PetId),
    /// Reservation record participating in the workflow.
    Reservation(reservation::Id),
    /// Incident record participating in the workflow.
    Incident(IncidentId),
    /// Approval decision record participating in audit history.
    Approval(approval::Id),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Actor that performed or is accountable for an audited action.
pub enum ActorRef {
    /// Customer record participating in the workflow.
    Customer(CustomerId),
    /// Staff id retained from source records for staff review, safety gates, and workflow joins.
    Staff {
        /// Staff id attached to this variant for reviewers and adapters.
        staff_id: StaffId,
    },
    /// Manager id retained from source records for staff review, safety gates, and workflow joins.
    Manager {
        /// Manager id attached to this variant for reviewers and adapters.
        manager_id: ManagerId,
    },
    /// System state or source category preserved for normalized resort records.
    System,
    /// Workflow retained from source records for staff review, safety gates, and workflow joins.
    Agent {
        /// Workflow attached to this variant for reviewers and adapters.
        workflow: agent::Name,
    },
}
