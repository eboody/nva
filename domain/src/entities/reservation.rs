//! Reservation vocabulary and checked reservation aggregates.

use chrono::{DateTime, Utc};
use serde::Deserializer;

use super::{
    customer::PortalProvider,
    identifiers::{CustomerId, LocationId, PetId},
};
use crate::{
    payment::{self, Deposit},
    policy,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use uuid::Uuid;

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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Reservation record tying customer, pet, service, status, deposit, add-ons, and safety stops together.
pub struct Reservation {
    /// Id retained from source records for staff review, safety gates, and workflow joins.
    id: Id,
    /// Location id retained from source records for staff review, safety gates, and workflow joins.
    location_id: LocationId,
    /// Customer id retained from source records for staff review, safety gates, and workflow joins.
    customer_id: CustomerId,
    /// Pet ids retained from source records for staff review, safety gates, and workflow joins.
    pet_party: PetParty,
    /// Requested service that drives scheduling and labor estimates.
    service: ServiceKind,
    /// Status retained from source records for staff review, safety gates, and workflow joins.
    status: Status,
    /// Starts at retained from source records for staff review, safety gates, and workflow joins.
    stay_interval: StayInterval,
    /// Ends at retained from source records for staff review, safety gates, and workflow joins.
    deposit: Option<Deposit>,
    /// Source retained from source records for staff review, safety gates, and workflow joins.
    source: Source,
    /// Requested add ons retained from source records for staff review, safety gates, and workflow joins.
    requested_add_ons: Vec<AddOn>,
    /// Hard stops retained from source records for staff review, safety gates, and workflow joins.
    hard_stops: Vec<HardStop>,
}

#[derive(Serialize, Deserialize)]
struct RawReservation {
    id: Id,
    location_id: LocationId,
    customer_id: CustomerId,
    pet_ids: Vec<PetId>,
    service: ServiceKind,
    status: Status,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    deposit: Option<Deposit>,
    source: Source,
    requested_add_ons: Vec<AddOn>,
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
    fn try_from_persisted(raw: RawReservation) -> Result<Self> {
        let stay_interval = StayInterval::try_new(raw.starts_at, raw.ends_at)?;
        let pet_party = PetParty::try_new(raw.pet_ids)?;
        if raw.hard_stops.contains(&HardStop::DepositRequired)
            && !raw
                .deposit
                .as_ref()
                .is_some_and(payment::Deposit::requires_collection)
        {
            return Err(Error::DepositRequiredHardStopNeedsCollectibleDeposit);
        }
        if matches!(
            raw.status,
            Status::Cancelled | Status::Rejected | Status::CheckedOut
        ) && !raw.hard_stops.is_empty()
        {
            return Err(Error::TerminalReservationCannotCarryActiveHardStops);
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

    /// Reservation identifier used by workflow, storage, and review joins.
    pub fn id(&self) -> Id {
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
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Checked reservation stay interval.
    pub fn stay_interval(&self) -> StayInterval {
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
    pub fn source(&self) -> &Source {
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
