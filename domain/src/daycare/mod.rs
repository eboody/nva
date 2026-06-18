//! Daycare service contracts for front-desk throughput, safe play, and package review.
//!
//! The module keeps care mode, eligibility, staffing ratios, and package policy explicit so
//! automated recommendations reduce check-in labor without bypassing staff review:
//!
//! ```
//! use domain::daycare;
//!
//! let contract = daycare::Contract::standard_petsuites();
//! assert!(contract.requires_staff_review_before_group_play());
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
        /// Human-readable name used in daycare workflows.
        pub struct $name($primitive);

        impl $name {
            /// Promotes boundary input into a validated daycare domain value.
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            /// Exposes the validated scalar for serialization and adapter boundaries.
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
        /// Validation failures returned by daycare domain constructors.
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
/// Domain vocabulary for service variant decisions in daycare workflows.
pub enum ServiceVariant {
    /// All day play daycare attendance, eligibility, coverage, or package signal.
    AllDayPlay,
    /// Half day play daycare attendance, eligibility, coverage, or package signal.
    HalfDayPlay,
    /// Daytime boarding care with lodging-style supervision.
    DayBoarding,
    /// Day play plus room daycare attendance, eligibility, coverage, or package signal.
    DayPlayPlusRoom,
    /// Cat individual playtime daycare attendance, eligibility, coverage, or package signal.
    CatIndividualPlaytime,
}

impl ServiceVariant {
    /// Returns this daycare value's care mode.
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
/// Domain vocabulary for care mode decisions in daycare workflows.
pub enum CareMode {
    /// Dog group play daycare attendance, eligibility, coverage, or package signal.
    DogGroupPlay,
    /// Dog individual day boarding daycare attendance, eligibility, coverage, or package signal.
    DogIndividualDayBoarding,
    /// Dog hybrid play and room daycare attendance, eligibility, coverage, or package signal.
    DogHybridPlayAndRoom,
    /// Cat individual enrichment daycare attendance, eligibility, coverage, or package signal.
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
/// Domain vocabulary for attendance policy decisions in daycare workflows.
pub enum AttendancePolicy {
    /// Drop in allowed daycare attendance, eligibility, coverage, or package signal.
    DropInAllowed,
    /// Reservation required daycare attendance, eligibility, coverage, or package signal.
    ReservationRequired,
    /// Capacity managed waitlist daycare attendance, eligibility, coverage, or package signal.
    CapacityManagedWaitlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for package policy decisions in daycare workflows.
pub enum PackagePolicy {
    /// Pay per visit daycare attendance, eligibility, coverage, or package signal.
    PayPerVisit,
    /// Visits fact promoted into this daycare contract.
    PrepaidPasses {
        /// Visits carried by this variant.
        visits: PackageVisits,
    },
    /// Membership daycare attendance, eligibility, coverage, or package signal.
    Membership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Typed staff pet ratio domain value that keeps raw primitives out of daycare workflows.
pub struct StaffPetRatio {
    staff: StaffCount,
    pets: PetCount,
}
impl StaffPetRatio {
    /// Assembles this daycare value from already-validated domain parts.
    pub const fn new(staff: StaffCount, pets: PetCount) -> Self {
        Self { staff, pets }
    }
    /// Returns this daycare value's pets per staff.
    pub const fn pets_per_staff(&self) -> PetCount {
        self.pets
    }
    /// Returns this daycare value's staff.
    pub const fn staff(&self) -> StaffCount {
        self.staff
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for group assignment rule decisions in daycare workflows.
pub enum GroupAssignmentRule {
    /// Temperament and size matched daycare attendance, eligibility, coverage, or package signal.
    TemperamentAndSizeMatched,
    /// Individual play only daycare attendance, eligibility, coverage, or package signal.
    IndividualPlayOnly,
    /// Senior or low energy group daycare attendance, eligibility, coverage, or package signal.
    SeniorOrLowEnergyGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for eligibility requirement decisions in daycare workflows.
pub enum EligibilityRequirement {
    /// Temperament assessment daycare attendance, eligibility, coverage, or package signal.
    TemperamentAssessment,
    /// Vaccines current daycare attendance, eligibility, coverage, or package signal.
    VaccinesCurrent,
    /// Spay neuter for group play daycare attendance, eligibility, coverage, or package signal.
    SpayNeuterForGroupPlay,
    /// Staff ratio available daycare attendance, eligibility, coverage, or package signal.
    StaffRatioAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Typed contract domain value that keeps raw primitives out of daycare workflows.
pub struct Contract {
    /// Attendance fact promoted into this daycare contract.
    pub attendance: AttendancePolicy,
    /// Package fact promoted into this daycare contract.
    pub package: PackagePolicy,
    /// Ratio fact promoted into this daycare contract.
    pub ratio: StaffPetRatio,
    /// Group assignment fact promoted into this daycare contract.
    pub group_assignment: GroupAssignmentRule,
    /// Incident fact promoted into this daycare contract.
    pub incident: incident::Policy,
    #[builder(default)]
    /// Eligibility fact promoted into this daycare contract.
    pub eligibility: Vec<EligibilityRequirement>,
}

impl Contract {
    /// Returns the requires staff review before group play for this daycare value.
    pub fn requires_staff_review_before_group_play(&self) -> bool {
        self.eligibility
            .contains(&EligibilityRequirement::TemperamentAssessment)
            || matches!(
                self.group_assignment,
                GroupAssignmentRule::TemperamentAndSizeMatched
            )
    }
    /// Returns the standard petsuites for this daycare value.
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
