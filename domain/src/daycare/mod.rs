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
        pub struct $name($primitive);

        impl $name {
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

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
        pub enum $error {
            #[error($message)]
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
pub enum ServiceVariant {
    AllDayPlay,
    HalfDayPlay,
    DayBoarding,
    DayPlayPlusRoom,
    CatIndividualPlaytime,
}

impl ServiceVariant {
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
pub enum CareMode {
    DogGroupPlay,
    DogIndividualDayBoarding,
    DogHybridPlayAndRoom,
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
pub enum AttendancePolicy {
    DropInAllowed,
    ReservationRequired,
    CapacityManagedWaitlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackagePolicy {
    PayPerVisit,
    PrepaidPasses { visits: PackageVisits },
    Membership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffPetRatio {
    staff: StaffCount,
    pets: PetCount,
}
impl StaffPetRatio {
    pub const fn new(staff: StaffCount, pets: PetCount) -> Self {
        Self { staff, pets }
    }
    pub const fn pets_per_staff(&self) -> PetCount {
        self.pets
    }
    pub const fn staff(&self) -> StaffCount {
        self.staff
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupAssignmentRule {
    TemperamentAndSizeMatched,
    IndividualPlayOnly,
    SeniorOrLowEnergyGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibilityRequirement {
    TemperamentAssessment,
    VaccinesCurrent,
    SpayNeuterForGroupPlay,
    StaffRatioAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Contract {
    pub attendance: AttendancePolicy,
    pub package: PackagePolicy,
    pub ratio: StaffPetRatio,
    pub group_assignment: GroupAssignmentRule,
    pub incident: incident::Policy,
    #[builder(default)]
    pub eligibility: Vec<EligibilityRequirement>,
}

impl Contract {
    pub fn requires_staff_review_before_group_play(&self) -> bool {
        self.eligibility
            .contains(&EligibilityRequirement::TemperamentAssessment)
            || matches!(
                self.group_assignment,
                GroupAssignmentRule::TemperamentAndSizeMatched
            )
    }
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
