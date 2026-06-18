use bon::Builder;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, LocationId, PetId, StaffId};

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        /// Human-readable name used in grooming workflows.
        pub struct $name($primitive);

        impl $name {
            /// Promotes boundary input into a validated grooming domain value.
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
        /// Validation failures returned by grooming domain constructors.
        pub enum $error {
            #[error($message)]
            /// Rejects zero where the pet-resort workflow requires a positive quantity.
            Zero,
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Grooming services and add-ons that affect scheduling, pricing, and labor time.
pub enum Service {
    /// Mini groom grooming service, assignment, estimate, or review signal.
    MiniGroom,
    /// Full groom grooming service, assignment, estimate, or review signal.
    FullGroom,
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Full bath grooming service, assignment, estimate, or review signal.
    FullBath,
    /// Premium bath grooming service, assignment, estimate, or review signal.
    PremiumBath,
    /// Nail trim grooming service, assignment, estimate, or review signal.
    NailTrim,
    /// Nail dremel grooming service, assignment, estimate, or review signal.
    NailDremel,
    /// Ear cleaning grooming service, assignment, estimate, or review signal.
    EarCleaning,
    /// Coat skin specific product grooming service, assignment, estimate, or review signal.
    CoatSkinSpecificProduct,
    /// First time grooming offer grooming service, assignment, estimate, or review signal.
    FirstTimeGroomingOffer,
}

positive_scalar!(
    AppointmentMinutes,
    u16,
    AppointmentMinutesError,
    "grooming appointment estimate requires at least one minute"
);

/// Calendar boundary for grooming contracts.
pub mod calendar {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Groomer-assignment policies used when booking grooming work.
    pub enum Policy {
        /// Any qualified groomer grooming service, assignment, estimate, or review signal.
        AnyQualifiedGroomer,
        /// Groomer specific grooming service, assignment, estimate, or review signal.
        GroomerSpecific,
        /// First available with manager override grooming service, assignment, estimate, or review signal.
        FirstAvailableWithManagerOverride,
    }
}
/// Breed coat boundary for grooming contracts.
pub mod breed_coat {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Breed and coat groupings used to estimate grooming labor time.
    pub enum BreedCategory {
        /// Short coat grooming service, assignment, estimate, or review signal.
        ShortCoat,
        /// Double coat grooming service, assignment, estimate, or review signal.
        DoubleCoat,
        /// Doodle grooming service, assignment, estimate, or review signal.
        Doodle,
        /// Cat guest, using cat-specific policy and accommodation rules.
        Cat,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Coat condition signals that affect grooming time and review needs.
    pub enum CoatCondition {
        /// Maintained grooming service, assignment, estimate, or review signal.
        Maintained,
        /// Thick undercoat grooming service, assignment, estimate, or review signal.
        ThickUndercoat,
        /// Matted grooming service, assignment, estimate, or review signal.
        Matted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Estimated grooming duration derived from breed, coat, and service context.
    pub struct TimeEstimate {
        /// Breed category used when estimating grooming effort.
        pub breed: BreedCategory,
        /// Coat condition used when estimating grooming effort.
        pub coat: CoatCondition,
        minutes: AppointmentMinutes,
    }

    impl TimeEstimate {
        /// Assembles this grooming value from already-validated domain parts.
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

        /// Returns this grooming value's minutes.
        pub const fn minutes(&self) -> AppointmentMinutes {
            self.minutes
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Historical grooming notes that should be preserved for future visits.
pub enum HistoryRequirement {
    /// Keep service notes grooming service, assignment, estimate, or review signal.
    KeepServiceNotes,
    /// Keep style notes and photos grooming service, assignment, estimate, or review signal.
    KeepStyleNotesAndPhotos,
    /// Keep medical handling notes grooming service, assignment, estimate, or review signal.
    KeepMedicalHandlingNotes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Inputs required to estimate grooming time before scheduling.
pub struct EstimationRequest {
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Requested service that drives scheduling and labor estimates.
    pub service: Service,
    /// Breed category used when estimating grooming effort.
    pub breed: breed_coat::BreedCategory,
    /// Coat condition used when estimating grooming effort.
    pub coat: breed_coat::CoatCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence source used to explain a grooming time estimate.
pub enum EstimateBasis {
    /// Breed coat policy grooming service, assignment, estimate, or review signal.
    BreedCoatPolicy,
    /// Groomer history grooming service, assignment, estimate, or review signal.
    GroomerHistory,
    /// Location default grooming service, assignment, estimate, or review signal.
    LocationDefault,
    /// Provider default grooming service, assignment, estimate, or review signal.
    ProviderDefault,
    /// Manual staff override grooming service, assignment, estimate, or review signal.
    ManualStaffOverride,
    /// Ai suggested pending review grooming service, assignment, estimate, or review signal.
    AiSuggestedPendingReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Confidence level assigned to a grooming duration estimate.
pub enum EstimateConfidence {
    /// Estimate is reliable enough for normal scheduling.
    High,
    /// Estimate is usable but should be treated with moderate uncertainty.
    Medium,
    /// Estimate is uncertain and may require staff confirmation.
    Low,
    /// Estimate confidence is unknown and must be reviewed.
    UnknownRequiresReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Staff review lane required before accepting a grooming estimate.
pub enum ReviewRequirement {
    /// No additional workflow gate is required.
    None,
    /// Staff review grooming service, assignment, estimate, or review signal.
    StaffReview,
    /// Groomer review grooming service, assignment, estimate, or review signal.
    GroomerReview,
    /// Manager review grooming service, assignment, estimate, or review signal.
    ManagerReview,
    /// Care review grooming service, assignment, estimate, or review signal.
    CareReview,
}

impl ReviewRequirement {
    /// Maps the grooming review lane to the workflow gate that must approve scheduling.
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
/// Typed duration estimate domain value that keeps raw primitives out of grooming workflows.
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

    /// Returns this grooming value's minutes.
    pub const fn minutes(&self) -> AppointmentMinutes {
        self.minutes
    }

    /// Returns this grooming value's basis.
    pub const fn basis(&self) -> EstimateBasis {
        self.basis
    }

    /// Returns this grooming value's confidence.
    pub const fn confidence(&self) -> EstimateConfidence {
        self.confidence
    }

    /// Returns this grooming value's review.
    pub const fn review(&self) -> ReviewRequirement {
        self.review
    }

    /// Maps the grooming review lane to the workflow gate that must approve scheduling.
    pub const fn calendar_execution_gate(&self) -> Option<crate::policy::ReviewGate> {
        self.review.calendar_execution_gate()
    }
}

#[derive(Debug, Clone, Default)]
/// Typed estimation policy domain value that keeps raw primitives out of grooming workflows.
pub struct EstimationPolicy;

impl EstimationPolicy {
    /// Returns the estimate for this grooming value.
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

/// No show boundary for grooming contracts.
pub mod no_show {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for rule decisions in grooming workflows.
    pub enum Rule {
        /// Note history only grooming service, assignment, estimate, or review signal.
        NoteHistoryOnly,
        /// Require deposit for rebooking grooming service, assignment, estimate, or review signal.
        RequireDepositForRebooking,
        /// Manager review before rebooking grooming service, assignment, estimate, or review signal.
        ManagerReviewBeforeRebooking,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Typed count domain value that keeps raw primitives out of grooming workflows.
    pub struct Count(u16);

    impl Count {
        /// Promotes boundary input into a validated grooming domain value.
        pub const fn try_new(value: u16) -> std::result::Result<Self, std::convert::Infallible> {
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Typed late cancel count domain value that keeps raw primitives out of grooming workflows.
    pub struct LateCancelCount(u16);

    impl LateCancelCount {
        /// Promotes boundary input into a validated grooming domain value.
        pub const fn try_new(value: u16) -> std::result::Result<Self, std::convert::Infallible> {
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed history domain value that keeps raw primitives out of grooming workflows.
    pub struct History {
        /// No shows fact promoted into this grooming contract.
        pub no_shows: Count,
        /// Late cancels fact promoted into this grooming contract.
        pub late_cancels: LateCancelCount,
    }

    impl History {
        /// Assembles this grooming value from already-validated domain parts.
        pub const fn new(no_shows: Count, late_cancels: LateCancelCount) -> Self {
            Self {
                no_shows,
                late_cancels,
            }
        }

        /// Returns this grooming value's repeat behavior count.
        pub const fn repeat_behavior_count(&self) -> u16 {
            self.no_shows.get().saturating_add(self.late_cancels.get())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for decision decisions in grooming workflows.
    pub enum Decision {
        /// Clear to rebook grooming service, assignment, estimate, or review signal.
        ClearToRebook,
        /// Gate fact promoted into this grooming contract.
        DepositRequired {
            /// Gate carried by this variant.
            gate: crate::policy::ReviewGate,
        },
        /// Gate fact promoted into this grooming contract.
        ManagerReviewRequired {
            /// Gate carried by this variant.
            gate: crate::policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed evaluation domain value that keeps raw primitives out of grooming workflows.
    pub struct Evaluation {
        /// Customer id fact promoted into this grooming contract.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// History fact promoted into this grooming contract.
        pub history: History,
    }

    #[derive(Debug, Clone)]
    /// Typed policy domain value that keeps raw primitives out of grooming workflows.
    pub struct Policy {
        rule: Rule,
    }

    impl Policy {
        /// Assembles this grooming value from already-validated domain parts.
        pub const fn new(rule: Rule) -> Self {
            Self { rule }
        }

        /// Returns the evaluate for this grooming value.
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

/// History boundary for grooming contracts.
pub mod history {
    use super::*;

    /// Style note boundary for grooming contracts.
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
    /// Domain vocabulary for care reference decisions in grooming workflows.
    pub enum CareReference {
        /// Sensitive skin product grooming service, assignment, estimate, or review signal.
        SensitiveSkinProduct,
        /// Medicated product requires review grooming service, assignment, estimate, or review signal.
        MedicatedProductRequiresReview,
        /// Handling or medical concern grooming service, assignment, estimate, or review signal.
        HandlingOrMedicalConcern,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for service outcome decisions in grooming workflows.
    pub enum ServiceOutcome {
        /// Completed grooming service, assignment, estimate, or review signal.
        Completed,
        /// No show grooming service, assignment, estimate, or review signal.
        NoShow,
        /// Late cancelled grooming service, assignment, estimate, or review signal.
        LateCancelled,
        /// Needs follow up grooming service, assignment, estimate, or review signal.
        NeedsFollowUp,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for approval state decisions in grooming workflows.
    pub enum ApprovalState {
        /// Draft grooming service, assignment, estimate, or review signal.
        Draft,
        /// Gate fact promoted into this grooming contract.
        ReviewRequired {
            /// Gate carried by this variant.
            gate: crate::policy::ReviewGate,
        },
        /// Groomer id fact promoted into this grooming contract.
        ApprovedByGroomer {
            /// Groomer id carried by this variant.
            groomer_id: StaffId,
        },
        /// Gate fact promoted into this grooming contract.
        Rejected {
            /// Gate carried by this variant.
            gate: crate::policy::ReviewGate,
        },
    }

    impl ApprovalState {
        /// Reports whether care-team review is needed before proceeding.
        pub const fn requires_review(&self) -> bool {
            matches!(self, Self::Draft | Self::ReviewRequired { .. })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
    /// Typed service history entry domain value that keeps raw primitives out of grooming workflows.
    pub struct ServiceHistoryEntry {
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Location id fact promoted into this grooming contract.
        pub location_id: LocationId,
        /// Requested service that drives scheduling and labor estimates.
        pub service: super::Service,
        /// Completed on fact promoted into this grooming contract.
        pub completed_on: NaiveDate,
        /// Outcome fact promoted into this grooming contract.
        pub outcome: ServiceOutcome,
        /// Approval fact promoted into this grooming contract.
        pub approval: ApprovalState,
        #[builder(default)]
        style_notes: Vec<style_note::StyleNote>,
        #[builder(default)]
        care_refs: Vec<CareReference>,
        duration: Option<AppointmentMinutes>,
    }

    impl ServiceHistoryEntry {
        /// Returns the style notes for this grooming value.
        pub fn style_notes(&self) -> &[style_note::StyleNote] {
            &self.style_notes
        }

        /// Returns the care refs for this grooming value.
        pub fn care_refs(&self) -> &[CareReference] {
            &self.care_refs
        }

        /// Returns this grooming value's duration.
        pub const fn duration(&self) -> Option<AppointmentMinutes> {
            self.duration
        }

        /// Reports whether care-team review is needed before proceeding.
        pub const fn requires_review(&self) -> bool {
            self.approval.requires_review() || !self.care_refs.is_empty()
        }
    }
}

/// Rebooking boundary for grooming contracts.
pub mod rebooking {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Typed cadence weeks domain value that keeps raw primitives out of grooming workflows.
    pub struct CadenceWeeks(u8);

    impl CadenceWeeks {
        /// Promotes boundary input into a validated grooming domain value.
        pub const fn try_new(value: u8) -> std::result::Result<Self, CadenceWeeksError> {
            if value == 0 {
                return Err(CadenceWeeksError::ZeroWeeks);
            }
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
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
    /// Domain vocabulary for cadence weeks error decisions in grooming workflows.
    pub enum CadenceWeeksError {
        #[error("grooming cadence requires at least one week")]
        /// Zero weeks grooming service, assignment, estimate, or review signal.
        ZeroWeeks,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Typed ordinary cadence weeks domain value that keeps raw primitives out of grooming workflows.
    pub struct OrdinaryCadenceWeeks(u8);

    impl OrdinaryCadenceWeeks {
        /// Promotes boundary input into a validated grooming domain value.
        pub const fn try_new(value: u8) -> std::result::Result<Self, OrdinaryCadenceWeeksError> {
            if value < 2 || value > 8 {
                return Err(OrdinaryCadenceWeeksError::OutsideOrdinaryGroomingBand);
            }
            Ok(Self(value))
        }

        /// Exposes the validated scalar for serialization and adapter boundaries.
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
    /// Domain vocabulary for ordinary cadence weeks error decisions in grooming workflows.
    pub enum OrdinaryCadenceWeeksError {
        #[error("ordinary grooming rebooking cadence must be between 2 and 8 weeks")]
        /// Outside ordinary grooming band grooming service, assignment, estimate, or review signal.
        OutsideOrdinaryGroomingBand,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for cadence decisions in grooming workflows.
    pub enum Cadence {
        /// Every weeks grooming service, assignment, estimate, or review signal.
        EveryWeeks(CadenceWeeks),
        /// As needed grooming service, assignment, estimate, or review signal.
        AsNeeded,
        /// Groomer recommended grooming service, assignment, estimate, or review signal.
        GroomerRecommended,
        /// Provider role or status could not be mapped confidently.
        Unknown,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized reservation states observed during source-data ingestion.
    pub enum Status {
        /// Due later grooming service, assignment, estimate, or review signal.
        DueLater,
        /// Due now grooming service, assignment, estimate, or review signal.
        DueNow,
        /// Overdue grooming service, assignment, estimate, or review signal.
        Overdue,
        /// Needs groomer recommendation grooming service, assignment, estimate, or review signal.
        NeedsGroomerRecommendation,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for rationale decisions in grooming workflows.
    pub enum Rationale {
        /// Last completed service cadence grooming service, assignment, estimate, or review signal.
        LastCompletedServiceCadence,
        /// No completed history grooming service, assignment, estimate, or review signal.
        NoCompletedHistory,
        /// Groomer recommended cadence required grooming service, assignment, estimate, or review signal.
        GroomerRecommendedCadenceRequired,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed recommendation domain value that keeps raw primitives out of grooming workflows.
    pub struct Recommendation {
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Due on fact promoted into this grooming contract.
        pub due_on: Option<NaiveDate>,
        /// Status fact promoted into this grooming contract.
        pub status: Status,
        /// Rationale fact promoted into this grooming contract.
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Default)]
    /// Typed policy domain value that keeps raw primitives out of grooming workflows.
    pub struct Policy;

    impl Policy {
        /// Returns the recommend from history for this grooming value.
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

/// Reminder boundary for grooming contracts.
pub mod reminder {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for rule decisions in grooming workflows.
    pub enum Rule {
        /// One week before grooming service, assignment, estimate, or review signal.
        OneWeekBefore,
        /// Forty eight hours before grooming service, assignment, estimate, or review signal.
        FortyEightHoursBefore,
        /// Morning of grooming service, assignment, estimate, or review signal.
        MorningOf,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for kind decisions in grooming workflows.
    pub enum Kind {
        /// Appointment confirmation grooming service, assignment, estimate, or review signal.
        AppointmentConfirmation,
        /// Prep instructions grooming service, assignment, estimate, or review signal.
        PrepInstructions,
        /// Morning of grooming service, assignment, estimate, or review signal.
        MorningOf,
        /// Rebooking due grooming service, assignment, estimate, or review signal.
        RebookingDue,
        /// Lapsed cadence winback grooming service, assignment, estimate, or review signal.
        LapsedCadenceWinback,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for consent decisions in grooming workflows.
    pub enum Consent {
        /// Granted grooming service, assignment, estimate, or review signal.
        Granted,
        /// Not granted grooming service, assignment, estimate, or review signal.
        NotGranted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Domain vocabulary for send boundary decisions in grooming workflows.
    pub enum SendBoundary {
        /// Draft requires approval grooming service, assignment, estimate, or review signal.
        DraftRequiresApproval,
        /// Ready for approved send grooming service, assignment, estimate, or review signal.
        ReadyForApprovedSend,
        /// Suppressed until consent grooming service, assignment, estimate, or review signal.
        SuppressedUntilConsent,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Typed plan domain value that keeps raw primitives out of grooming workflows.
    pub struct Plan {
        /// Customer id fact promoted into this grooming contract.
        pub customer_id: CustomerId,
        /// Kind fact promoted into this grooming contract.
        pub kind: Kind,
        boundary: SendBoundary,
    }

    impl Plan {
        /// Returns this grooming value's send boundary.
        pub const fn send_boundary(&self) -> SendBoundary {
            self.boundary
        }

        /// Returns this grooming value's customer message gate.
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
    /// Typed policy domain value that keeps raw primitives out of grooming workflows.
    pub struct Policy;

    impl Policy {
        /// Returns this grooming value's plan.
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
/// Typed contract domain value that keeps raw primitives out of grooming workflows.
pub struct Contract {
    /// Calendar fact promoted into this grooming contract.
    pub calendar: calendar::Policy,
    #[builder(default)]
    /// Time estimates fact promoted into this grooming contract.
    pub time_estimates: Vec<breed_coat::TimeEstimate>,
    /// No show fact promoted into this grooming contract.
    pub no_show: no_show::Rule,
    /// Rebooking fact promoted into this grooming contract.
    pub rebooking: rebooking::Cadence,
    #[builder(default)]
    /// Reminders fact promoted into this grooming contract.
    pub reminders: Vec<reminder::Rule>,
    /// History fact promoted into this grooming contract.
    pub history: HistoryRequirement,
}

impl Contract {
    /// Returns the requires deposit after no show for this grooming value.
    pub fn requires_deposit_after_no_show(&self) -> bool {
        matches!(
            self.no_show,
            no_show::Rule::RequireDepositForRebooking | no_show::Rule::ManagerReviewBeforeRebooking
        )
    }
    /// Returns the standard petsuites for this grooming value.
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
