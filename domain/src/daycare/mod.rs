//! Daycare service rules for front-desk throughput, safe play, and package review.
//!
//! Operator summary: this module supports check-in lane, coverage, group-play eligibility,
//! playgroup assignment, incident-restriction, and package-opportunity decisions for
//! front-desk, care-team, and manager queues. It reduces repeated manual lookups of
//! attendance policy, package state, staff-to-pet ratio, temperament/vaccine readiness,
//! incident status, and customer-message readiness.
//!
//! Use it when the operational question is "can this pet enter the daycare flow safely,
//! where should staff route the work, and what review gate still stands between a draft
//! recommendation and a live action?" The next step is to start with the location rules for
//! location policy, then follow the child modules for the queue you are working: `front_desk`
//! for check-in, `eligibility` for play clearance, `coverage` for ratio risk,
//! `assignment` for playgroup fit, `incident` for restrictions, or `package_opportunity`
//! for billing/customer-message review.
//!
//! It does not authorize live admission, provider writes, reservation mutation, payment
//! collection, package enrollment, customer sends, incident reinstatement, or manager
//! overrides. Reservation/pet/source provenance, staff roster facts, package/payment state,
//! and the location daycare rules remain authoritative inputs; review gates such as
//! behavior review, medical/document review, manager approval, customer-message approval,
//! and billing review protect pets, customers, and staff before any side effect.
//!
//! The module keeps care mode, eligibility, staffing ratios, and package policy explicit so
//! automated recommendations reduce check-in labor without bypassing staff review:
//!
//! ```
//! use domain::daycare;
//!
//! let rules = daycare::Contract::standard_petsuites();
//! assert!(rules.requires_staff_review_before_group_play());
//! assert_eq!(
//!     daycare::ServiceVariant::DayBoarding.care_mode(),
//!     daycare::CareMode::DogIndividualDayBoarding,
//! );
//! ```

use bon::Builder;
use chrono::NaiveDate;
use nutype::nutype;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, PetId};

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        /// Positive daycare quantity used where zero would hide real staffing, pet-count, queue, or package volume.
        pub struct $name($primitive);

        impl $name {
            /// Rejects impossible daycare counts before they affect group-play capacity, staffing ratios, eligibility queues, or package balances.
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            /// Returns the daycare count used by package, ratio, eligibility, or coverage calculations.
            pub const fn get(self) -> $primitive {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::try_new(<$primitive>::deserialize(deserializer)?)
                    .map_err(serde::de::Error::custom)
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
        /// Validation failure returned when a required positive daycare scalar is zero.
        pub enum $error {
            #[error($message)]
            /// Rejects zero where the pet-resort workflow requires a positive quantity.
            Zero,
        }
    };
}

positive_scalar!(
    PackageVisits,
    u16,
    PackageVisitsError,
    "daycare packages require at least one visit"
);
positive_scalar!(
    StaffCount,
    u16,
    StaffCountError,
    "daycare ratio requires at least one staff member"
);
positive_scalar!(
    PetCount,
    u16,
    PetCountError,
    "daycare ratio requires at least one pet"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Daycare service variant requested by a customer or reservation workflow.
pub enum ServiceVariant {
    /// Full-day dog group-play service requiring eligibility and staffing-ratio checks.
    AllDayPlay,
    /// Partial-day dog group-play service requiring eligibility and staffing-ratio checks.
    HalfDayPlay,
    /// Daytime boarding care with lodging-style supervision.
    DayBoarding,
    /// Hybrid daycare offering that combines play with room-based rest or supervision.
    DayPlayPlusRoom,
    /// Cat enrichment service that remains separate from dog group-play eligibility rules.
    CatIndividualPlaytime,
}

impl ServiceVariant {
    /// Maps the customer-facing service variant to the care mode used by eligibility and staffing policy.
    pub const fn care_mode(self) -> CareMode {
        match self {
            Self::AllDayPlay | Self::HalfDayPlay => CareMode::DogGroupPlay,
            Self::DayBoarding => CareMode::DogIndividualDayBoarding,
            Self::DayPlayPlusRoom => CareMode::DogHybridPlayAndRoom,
            Self::CatIndividualPlaytime => CareMode::CatIndividualEnrichment,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Operational care mode that determines whether group-play, individual care, or cat enrichment rules apply.
pub enum CareMode {
    /// Dog group-play mode requiring temperament, vaccine, and staffing-ratio clearance.
    DogGroupPlay,
    /// Individual dog supervision mode for pets not suited to group play or needing quieter care.
    DogIndividualDayBoarding,
    /// Mixed dog care mode combining group-play eligibility with room-based supervision.
    DogHybridPlayAndRoom,
    /// Individual cat enrichment mode outside dog playgroup assignment.
    CatIndividualEnrichment,
}

pub mod incident;

pub mod coverage;

pub mod eligibility;

pub mod assignment;

pub mod attendance;

pub mod package_opportunity;

pub mod front_desk;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Attendance policy controlling whether daycare check-in is drop-in, reserved, or waitlisted.
pub enum AttendancePolicy {
    /// Staff may accept unscheduled daycare arrivals if other gates are clear.
    DropInAllowed,
    /// Staff should require a reservation before admitting daycare attendance.
    ReservationRequired,
    /// Capacity constraints require staff to route new daycare demand through a waitlist.
    CapacityManagedWaitlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Daycare payment/package model used for collection and recommendation workflows.
pub enum PackagePolicy {
    /// Customer pays each visit without a prepaid package or membership covering attendance.
    PayPerVisit,
    /// Prepaid visit count available to apply against daycare attendance.
    PrepaidPasses {
        /// Prepaid daycare visits available before billing or package review is needed.
        visits: PackageVisits,
    },
    /// Customer has a membership covering the daycare attendance path.
    Membership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Staff-to-pet ratio used to decide whether group-play coverage is sufficient.
pub struct StaffPetRatio {
    staff: StaffCount,
    pets: PetCount,
}
impl StaffPetRatio {
    /// Creates the daycare value from validated domain parts without trusting raw source primitives.
    pub const fn new(staff: StaffCount, pets: PetCount) -> Self {
        Self { staff, pets }
    }
    /// Returns the allowed pet count per staff member for coverage checks.
    pub const fn pets_per_staff(&self) -> PetCount {
        self.pets
    }
    /// Returns the staff side of the ratio used by coverage checks.
    pub const fn staff(&self) -> StaffCount {
        self.staff
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Rule used to choose or restrict daycare group assignment.
pub enum GroupAssignmentRule {
    /// Assign pets only to groups matched by temperament and size evidence.
    TemperamentAndSizeMatched,
    /// Restrict the pet to individual play instead of group assignment.
    IndividualPlayOnly,
    /// Route the pet to a calmer group suited to senior or low-energy needs.
    SeniorOrLowEnergyGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence requirements that must be satisfied before daycare care modes proceed.
pub enum EligibilityRequirement {
    /// Current temperament assessment is required before group-play admission.
    TemperamentAssessment,
    /// Vaccine proof must be current before daycare admission.
    VaccinesCurrent,
    /// Spay/neuter status must be reviewed for dog group-play eligibility.
    SpayNeuterForGroupPlay,
    /// Staffing coverage must satisfy the configured ratio before attendance proceeds.
    StaffRatioAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Daycare service-line ruleset combining attendance, package, ratio, assignment, incident, and eligibility policy.
pub struct Contract {
    /// Attendance gate controlling reservations, drop-ins, and waitlist routing.
    pub attendance: AttendancePolicy,
    /// Package or payment model used for front-desk collection and sales opportunities.
    pub package: PackagePolicy,
    /// Staff-to-pet ratio that defines safe coverage for daycare operations.
    pub ratio: StaffPetRatio,
    /// Assignment rule that protects playgroup fit and care safety.
    pub group_assignment: GroupAssignmentRule,
    /// Incident handling policy that can require manager review or customer notice.
    pub incident: incident::Policy,
    #[builder(default)]
    /// Eligibility requirements that must be evidenced before group-play automation proceeds.
    pub eligibility: Vec<EligibilityRequirement>,
}

impl Contract {
    /// Reports whether these daycare rules require staff review before admitting a pet to group play.
    pub fn requires_staff_review_before_group_play(&self) -> bool {
        self.eligibility
            .contains(&EligibilityRequirement::TemperamentAssessment)
            || matches!(
                self.group_assignment,
                GroupAssignmentRule::TemperamentAndSizeMatched
            )
    }
    /// Builds the baseline PetSuites-style daycare rules used by examples and tests.
    pub fn standard_petsuites() -> Self {
        Self::builder()
            .attendance(AttendancePolicy::ReservationRequired)
            .package(PackagePolicy::PrepaidPasses {
                visits: PackageVisits::try_new(5).unwrap(),
            })
            .ratio(StaffPetRatio::new(
                StaffCount::try_new(1).unwrap(),
                PetCount::try_new(12).unwrap(),
            ))
            .group_assignment(GroupAssignmentRule::TemperamentAndSizeMatched)
            .incident(incident::Policy::ManagerReviewAndCustomerNotice)
            .eligibility(vec![
                EligibilityRequirement::TemperamentAssessment,
                EligibilityRequirement::VaccinesCurrent,
            ])
            .build()
    }
}
