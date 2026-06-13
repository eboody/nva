use bon::Builder;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, LocationId, PetId, StaffId};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Service {
    MiniGroom,
    FullGroom,
    ExitBath,
    FullBath,
    PremiumBath,
    NailTrim,
    NailDremel,
    EarCleaning,
    CoatSkinSpecificProduct,
    FirstTimeGroomingOffer,
}

positive_scalar!(
    AppointmentMinutes,
    u16,
    AppointmentMinutesError,
    "grooming appointment estimate requires at least one minute"
);

pub mod calendar {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Policy {
        AnyQualifiedGroomer,
        GroomerSpecific,
        FirstAvailableWithManagerOverride,
    }
}
pub mod breed_coat {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum BreedCategory {
        ShortCoat,
        DoubleCoat,
        Doodle,
        Cat,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CoatCondition {
        Maintained,
        ThickUndercoat,
        Matted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TimeEstimate {
        pub breed: BreedCategory,
        pub coat: CoatCondition,
        minutes: AppointmentMinutes,
    }

    impl TimeEstimate {
        pub const fn new(
            breed: BreedCategory,
            coat: CoatCondition,
            minutes: AppointmentMinutes,
        ) -> Self {
            Self {
                breed,
                coat,
                minutes,
            }
        }

        pub const fn minutes(&self) -> AppointmentMinutes {
            self.minutes
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryRequirement {
    KeepServiceNotes,
    KeepStyleNotesAndPhotos,
    KeepMedicalHandlingNotes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct EstimationRequest {
    pub pet_id: PetId,
    pub service: Service,
    pub breed: breed_coat::BreedCategory,
    pub coat: breed_coat::CoatCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstimateBasis {
    BreedCoatPolicy,
    GroomerHistory,
    LocationDefault,
    ProviderDefault,
    ManualStaffOverride,
    AiSuggestedPendingReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstimateConfidence {
    High,
    Medium,
    Low,
    UnknownRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewRequirement {
    None,
    StaffReview,
    GroomerReview,
    ManagerReview,
    CareReview,
}

impl ReviewRequirement {
    pub const fn calendar_execution_gate(self) -> Option<crate::policy::ReviewGate> {
        match self {
            Self::None => None,
            Self::StaffReview | Self::GroomerReview | Self::ManagerReview => {
                Some(crate::policy::ReviewGate::ManagerApproval)
            }
            Self::CareReview => Some(crate::policy::ReviewGate::MedicalDocumentReview),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationEstimate {
    minutes: AppointmentMinutes,
    basis: EstimateBasis,
    confidence: EstimateConfidence,
    review: ReviewRequirement,
}

impl DurationEstimate {
    const fn new(
        minutes: AppointmentMinutes,
        basis: EstimateBasis,
        confidence: EstimateConfidence,
        review: ReviewRequirement,
    ) -> Self {
        Self {
            minutes,
            basis,
            confidence,
            review,
        }
    }

    pub const fn minutes(&self) -> AppointmentMinutes {
        self.minutes
    }

    pub const fn basis(&self) -> EstimateBasis {
        self.basis
    }

    pub const fn confidence(&self) -> EstimateConfidence {
        self.confidence
    }

    pub const fn review(&self) -> ReviewRequirement {
        self.review
    }

    pub const fn calendar_execution_gate(&self) -> Option<crate::policy::ReviewGate> {
        self.review.calendar_execution_gate()
    }
}

#[derive(Debug, Clone, Default)]
pub struct EstimationPolicy;

impl EstimationPolicy {
    pub fn estimate(
        &self,
        request: EstimationRequest,
        history: &[history::ServiceHistoryEntry],
        contract: &Contract,
    ) -> DurationEstimate {
        if let Some(entry) = history
            .iter()
            .rev()
            .find(|entry| entry.pet_id == request.pet_id && entry.duration().is_some())
        {
            return DurationEstimate::new(
                entry.duration().expect("checked above"),
                EstimateBasis::GroomerHistory,
                EstimateConfidence::Medium,
                if entry.requires_review() {
                    ReviewRequirement::GroomerReview
                } else {
                    ReviewRequirement::None
                },
            );
        }

        let minutes = contract
            .time_estimates
            .iter()
            .find(|estimate| estimate.breed == request.breed && estimate.coat == request.coat)
            .or_else(|| {
                contract
                    .time_estimates
                    .iter()
                    .find(|estimate| estimate.breed == request.breed)
            })
            .map(breed_coat::TimeEstimate::minutes)
            .unwrap_or_else(|| {
                AppointmentMinutes::try_new(60).expect("default estimate is positive")
            });

        let review = match request.coat {
            breed_coat::CoatCondition::Matted => ReviewRequirement::GroomerReview,
            breed_coat::CoatCondition::Maintained | breed_coat::CoatCondition::ThickUndercoat => {
                ReviewRequirement::None
            }
        };
        let confidence = if matches!(review, ReviewRequirement::None) {
            EstimateConfidence::High
        } else {
            EstimateConfidence::Medium
        };

        DurationEstimate::new(minutes, EstimateBasis::BreedCoatPolicy, confidence, review)
    }
}

pub mod no_show {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Rule {
        NoteHistoryOnly,
        RequireDepositForRebooking,
        ManagerReviewBeforeRebooking,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Count(u16);

    impl Count {
        pub const fn try_new(value: u16) -> std::result::Result<Self, std::convert::Infallible> {
            Ok(Self(value))
        }

        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct LateCancelCount(u16);

    impl LateCancelCount {
        pub const fn try_new(value: u16) -> std::result::Result<Self, std::convert::Infallible> {
            Ok(Self(value))
        }

        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct History {
        pub no_shows: Count,
        pub late_cancels: LateCancelCount,
    }

    impl History {
        pub const fn new(no_shows: Count, late_cancels: LateCancelCount) -> Self {
            Self {
                no_shows,
                late_cancels,
            }
        }

        pub const fn repeat_behavior_count(&self) -> u16 {
            self.no_shows.get().saturating_add(self.late_cancels.get())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Decision {
        ClearToRebook,
        DepositRequired { gate: crate::policy::ReviewGate },
        ManagerReviewRequired { gate: crate::policy::ReviewGate },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Evaluation {
        pub customer_id: CustomerId,
        pub pet_id: PetId,
        pub history: History,
    }

    #[derive(Debug, Clone)]
    pub struct Policy {
        rule: Rule,
    }

    impl Policy {
        pub const fn new(rule: Rule) -> Self {
            Self { rule }
        }

        pub fn evaluate(
            &self,
            customer_id: CustomerId,
            pet_id: PetId,
            history: History,
        ) -> Decision {
            let _evaluation = Evaluation {
                customer_id,
                pet_id,
                history,
            };
            match self.rule {
                Rule::NoteHistoryOnly => Decision::ClearToRebook,
                Rule::RequireDepositForRebooking if history.repeat_behavior_count() > 0 => {
                    Decision::DepositRequired {
                        gate: crate::policy::ReviewGate::RefundOrDepositException,
                    }
                }
                Rule::RequireDepositForRebooking => Decision::ClearToRebook,
                Rule::ManagerReviewBeforeRebooking => Decision::ManagerReviewRequired {
                    gate: crate::policy::ReviewGate::ManagerApproval,
                },
            }
        }
    }
}

pub mod history {
    use super::*;

    pub mod style_note {
        use nutype::nutype;

        #[nutype(
            sanitize(trim),
            validate(not_empty, len_char_max = 500),
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
        pub struct StyleNote(String);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CareReference {
        SensitiveSkinProduct,
        MedicatedProductRequiresReview,
        HandlingOrMedicalConcern,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ServiceOutcome {
        Completed,
        NoShow,
        LateCancelled,
        NeedsFollowUp,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ApprovalState {
        Draft,
        ReviewRequired { gate: crate::policy::ReviewGate },
        ApprovedByGroomer { groomer_id: StaffId },
        Rejected { gate: crate::policy::ReviewGate },
    }

    impl ApprovalState {
        pub const fn requires_review(&self) -> bool {
            matches!(self, Self::Draft | Self::ReviewRequired { .. })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    pub struct ServiceHistoryEntry {
        pub pet_id: PetId,
        pub location_id: LocationId,
        pub service: super::Service,
        pub completed_on: NaiveDate,
        pub outcome: ServiceOutcome,
        pub approval: ApprovalState,
        #[builder(default)]
        style_notes: Vec<style_note::StyleNote>,
        #[builder(default)]
        care_refs: Vec<CareReference>,
        duration: Option<AppointmentMinutes>,
    }

    impl ServiceHistoryEntry {
        pub fn style_notes(&self) -> &[style_note::StyleNote] {
            &self.style_notes
        }

        pub fn care_refs(&self) -> &[CareReference] {
            &self.care_refs
        }

        pub const fn duration(&self) -> Option<AppointmentMinutes> {
            self.duration
        }

        pub const fn requires_review(&self) -> bool {
            self.approval.requires_review() || !self.care_refs.is_empty()
        }
    }
}

pub mod rebooking {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    pub struct CadenceWeeks(u8);

    impl CadenceWeeks {
        pub const fn try_new(value: u8) -> std::result::Result<Self, CadenceWeeksError> {
            if value == 0 {
                return Err(CadenceWeeksError::ZeroWeeks);
            }
            Ok(Self(value))
        }

        pub const fn get(self) -> u8 {
            self.0
        }
    }

    impl<'de> Deserialize<'de> for CadenceWeeks {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum CadenceWeeksError {
        #[error("grooming cadence requires at least one week")]
        ZeroWeeks,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    pub struct OrdinaryCadenceWeeks(u8);

    impl OrdinaryCadenceWeeks {
        pub const fn try_new(value: u8) -> std::result::Result<Self, OrdinaryCadenceWeeksError> {
            if value < 2 || value > 8 {
                return Err(OrdinaryCadenceWeeksError::OutsideOrdinaryGroomingBand);
            }
            Ok(Self(value))
        }

        pub const fn get(self) -> u8 {
            self.0
        }
    }

    impl<'de> Deserialize<'de> for OrdinaryCadenceWeeks {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum OrdinaryCadenceWeeksError {
        #[error("ordinary grooming rebooking cadence must be between 2 and 8 weeks")]
        OutsideOrdinaryGroomingBand,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Cadence {
        EveryWeeks(CadenceWeeks),
        AsNeeded,
        GroomerRecommended,
        Unknown,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Status {
        DueLater,
        DueNow,
        Overdue,
        NeedsGroomerRecommendation,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Rationale {
        LastCompletedServiceCadence,
        NoCompletedHistory,
        GroomerRecommendedCadenceRequired,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Recommendation {
        pub pet_id: PetId,
        pub due_on: Option<NaiveDate>,
        pub status: Status,
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Default)]
    pub struct Policy;

    impl Policy {
        pub fn recommend_from_history(
            &self,
            pet_id: PetId,
            history: &[history::ServiceHistoryEntry],
            cadence: Cadence,
            today: NaiveDate,
        ) -> Recommendation {
            let Some(last_completed) = history
                .iter()
                .filter(|entry| entry.pet_id == pet_id)
                .filter(|entry| matches!(entry.outcome, history::ServiceOutcome::Completed))
                .max_by_key(|entry| entry.completed_on)
            else {
                return Recommendation {
                    pet_id,
                    due_on: None,
                    status: Status::NeedsGroomerRecommendation,
                    rationale: Rationale::NoCompletedHistory,
                };
            };

            let Cadence::EveryWeeks(weeks) = cadence else {
                return Recommendation {
                    pet_id,
                    due_on: None,
                    status: Status::NeedsGroomerRecommendation,
                    rationale: Rationale::GroomerRecommendedCadenceRequired,
                };
            };

            let due_on = last_completed
                .completed_on
                .checked_add_days(chrono::Days::new(u64::from(weeks.get()) * 7))
                .expect("bounded grooming cadence should fit chrono date range");
            let status = if today > due_on {
                Status::Overdue
            } else if today == due_on {
                Status::DueNow
            } else {
                Status::DueLater
            };

            Recommendation {
                pet_id,
                due_on: Some(due_on),
                status,
                rationale: Rationale::LastCompletedServiceCadence,
            }
        }
    }
}

pub mod reminder {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Rule {
        OneWeekBefore,
        FortyEightHoursBefore,
        MorningOf,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Kind {
        AppointmentConfirmation,
        PrepInstructions,
        MorningOf,
        RebookingDue,
        LapsedCadenceWinback,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Consent {
        Granted,
        NotGranted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum SendBoundary {
        DraftRequiresApproval,
        ReadyForApprovedSend,
        SuppressedUntilConsent,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Plan {
        pub customer_id: CustomerId,
        pub kind: Kind,
        boundary: SendBoundary,
    }

    impl Plan {
        pub const fn send_boundary(&self) -> SendBoundary {
            self.boundary
        }

        pub const fn customer_message_gate(&self) -> Option<crate::policy::ReviewGate> {
            match self.boundary {
                SendBoundary::DraftRequiresApproval => {
                    Some(crate::policy::ReviewGate::CustomerMessageApproval)
                }
                SendBoundary::ReadyForApprovedSend | SendBoundary::SuppressedUntilConsent => None,
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Policy;

    impl Policy {
        pub const fn plan(&self, customer_id: CustomerId, kind: Kind, consent: Consent) -> Plan {
            let boundary = match consent {
                Consent::Granted => SendBoundary::DraftRequiresApproval,
                Consent::NotGranted => SendBoundary::SuppressedUntilConsent,
            };
            Plan {
                customer_id,
                kind,
                boundary,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Contract {
    pub calendar: calendar::Policy,
    #[builder(default)]
    pub time_estimates: Vec<breed_coat::TimeEstimate>,
    pub no_show: no_show::Rule,
    pub rebooking: rebooking::Cadence,
    #[builder(default)]
    pub reminders: Vec<reminder::Rule>,
    pub history: HistoryRequirement,
}

impl Contract {
    pub fn requires_deposit_after_no_show(&self) -> bool {
        matches!(
            self.no_show,
            no_show::Rule::RequireDepositForRebooking | no_show::Rule::ManagerReviewBeforeRebooking
        )
    }
    pub fn standard_petsuites() -> Self {
        Self::builder()
            .calendar(calendar::Policy::GroomerSpecific)
            .time_estimates(vec![breed_coat::TimeEstimate::new(
                breed_coat::BreedCategory::Doodle,
                breed_coat::CoatCondition::Matted,
                AppointmentMinutes::try_new(180).unwrap(),
            )])
            .no_show(no_show::Rule::RequireDepositForRebooking)
            .rebooking(rebooking::Cadence::EveryWeeks(
                rebooking::CadenceWeeks::try_new(6).unwrap(),
            ))
            .reminders(vec![
                reminder::Rule::FortyEightHoursBefore,
                reminder::Rule::MorningOf,
            ])
            .history(HistoryRequirement::KeepStyleNotesAndPhotos)
            .build()
    }
}

/// Appointment-owned public vocabulary for grooming service requests.
pub mod appointment {
    pub use super::{EstimationRequest as Request, Service};
}

/// Duration-estimate decision vocabulary.
pub mod duration_estimate {
    pub use super::{
        AppointmentMinutes, AppointmentMinutesError, DurationEstimate, EstimateBasis,
        EstimateConfidence, EstimationPolicy as Policy, ReviewRequirement,
    };
}
