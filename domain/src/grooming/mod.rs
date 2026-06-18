//! Grooming service-line contracts for pet-resort labor planning, rebooking, reminders, and safe customer-facing grooming automation.
//!
//! This module models mini/full grooms, baths, coat/skin add-ons, groomer assignment, duration estimates, no-show consequences, service history, and reminder cadence as source-derived operational facts. AI or adapter code may draft estimates, reminders, and rebooking prompts through these types, but manager/groomer/care review gates preserve the boundary between recommendation and live scheduling or customer messaging.

use bon::Builder;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::{CustomerId, LocationId, PetId, StaffId};

macro_rules! positive_scalar {
    ($name:ident, $primitive:ty, $error:ident, $message:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        /// Positive grooming quantity used where a zero-minute appointment or zero-length operational value would create impossible schedule math.
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
/// Grooming services and add-ons that drive groomer calendar load, checkout upsells, duration estimates, and follow-up reminders.
pub enum Service {
    /// Mini groom request that typically consumes less groomer time but still needs coat/history context.
    MiniGroom,
    /// Full groom request that drives the heaviest groomer labor estimate and style-history review.
    FullGroom,
    /// Bath offered before departure from boarding.
    ExitBath,
    /// Full bath appointment that may stand alone or attach to daycare/boarding checkout.
    FullBath,
    /// Premium bath that can justify product/style-note capture and higher checkout value.
    PremiumBath,
    /// Nail trim add-on that affects short-slot grooming capacity.
    NailTrim,
    /// Nail Dremel add-on that should respect pet handling notes and appointment timing.
    NailDremel,
    /// Ear-cleaning add-on whose care sensitivity may require staff review before customer claims.
    EarCleaning,
    /// Coat/skin product add-on that should remain a product recommendation unless care review approves stronger claims.
    CoatSkinSpecificProduct,
    /// First-time grooming offer used to convert new/lapsed guests without bypassing scheduling constraints.
    FirstTimeGroomingOffer,
}

positive_scalar!(
    AppointmentMinutes,
    u16,
    AppointmentMinutesError,
    "grooming appointment estimate requires at least one minute"
);

/// Groomer-calendar policy boundary for assigning grooming work without inventing availability.
pub mod calendar {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Groomer-assignment policy used to decide whether a request can draft directly or needs manager/groomer review.
    pub enum Policy {
        /// Any qualified groomer may take the appointment if the schedule system shows capacity.
        AnyQualifiedGroomer,
        /// A specific groomer is required because of guest history, owner request, or service complexity.
        GroomerSpecific,
        /// First-available assignment is allowed only with a manager override when ordinary matching cannot satisfy demand.
        FirstAvailableWithManagerOverride,
    }
}
/// Breed/coat boundary for converting pet profile facts into labor-time estimates.
pub mod breed_coat {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Breed and coat groupings used to estimate grooming labor time.
    pub enum BreedCategory {
        /// Short-coat category with lower expected grooming labor when no history indicates otherwise.
        ShortCoat,
        /// Double-coat category that may require extra drying/deshedding time.
        DoubleCoat,
        /// Doodle or similar coat category where matting/style history often changes the estimate.
        Doodle,
        /// Cat guest, using cat-specific policy and accommodation rules.
        Cat,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Coat condition signals that affect grooming time and review needs.
    pub enum CoatCondition {
        /// Maintained coat condition suitable for standard estimates.
        Maintained,
        /// Thick undercoat condition that increases labor estimate and may alter product recommendations.
        ThickUndercoat,
        /// Matted coat condition that requires groomer review before accepting a duration estimate.
        Matted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Duration estimate input derived from breed and coat facts for groomer calendar planning.
    pub struct TimeEstimate {
        /// Breed/coat class used to translate pet profile data into groomer labor demand.
        pub breed: BreedCategory,
        /// Coat condition that can raise confidence risk or trigger groomer review.
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

        /// Returns the minutes evidence recorded on this grooming contract.
        pub const fn minutes(&self) -> AppointmentMinutes {
            self.minutes
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Service-history retention requirement that protects rebooking quality and safe handling across visits.
pub enum HistoryRequirement {
    /// Preserve service notes so future estimates can cite source history rather than invent timing.
    KeepServiceNotes,
    /// Preserve style notes/photos so groomers can reproduce customer preferences at the next cadence.
    KeepStyleNotesAndPhotos,
    /// Preserve medical or handling notes and route sensitive interpretation through care review.
    KeepMedicalHandlingNotes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Source-derived request used to estimate grooming duration before a schedule mutation is proposed.
pub struct EstimationRequest {
    /// Pet receiving the grooming or care service.
    pub pet_id: PetId,
    /// Requested service that drives scheduling and labor estimates.
    pub service: Service,
    /// Breed/coat class used to translate pet profile data into groomer labor demand.
    pub breed: breed_coat::BreedCategory,
    /// Coat condition that can raise confidence risk or trigger groomer review.
    pub coat: breed_coat::CoatCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Evidence basis that explains why a grooming duration was chosen for scheduling review.
pub enum EstimateBasis {
    /// Estimate came from the location contract for breed/coat combinations.
    BreedCoatPolicy,
    /// Estimate came from prior groomer history for this pet.
    GroomerHistory,
    /// Estimate fell back to a location default when stronger source facts were unavailable.
    LocationDefault,
    /// Estimate came from provider defaults and should not override local policy silently.
    ProviderDefault,
    /// Estimate was overridden by staff and should be auditable as a human-entered fact.
    ManualStaffOverride,
    /// Estimate was suggested by automation and must remain pending review before schedule use.
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
/// Review lane that determines whether a grooming estimate may be used for calendar execution.
pub enum ReviewRequirement {
    /// No additional workflow gate is required.
    None,
    /// General staff review is required before this estimate becomes actionable.
    StaffReview,
    /// Groomer review is required because coat/history/service complexity affects labor time.
    GroomerReview,
    /// Manager review is required before accepting an exceptional estimate or schedule choice.
    ManagerReview,
    /// Care/medical-document review is required before acting on sensitive handling information.
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
/// Grooming duration decision with evidence, confidence, and the review gate needed before calendar use.
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

    /// Returns the minutes evidence recorded on this grooming contract.
    pub const fn minutes(&self) -> AppointmentMinutes {
        self.minutes
    }

    /// Returns the basis evidence recorded on this grooming contract.
    pub const fn basis(&self) -> EstimateBasis {
        self.basis
    }

    /// Returns the confidence evidence recorded on this grooming contract.
    pub const fn confidence(&self) -> EstimateConfidence {
        self.confidence
    }

    /// Returns the review evidence recorded on this grooming contract.
    pub const fn review(&self) -> ReviewRequirement {
        self.review
    }

    /// Maps the grooming review lane to the workflow gate that must approve scheduling.
    pub const fn calendar_execution_gate(&self) -> Option<crate::policy::ReviewGate> {
        self.review.calendar_execution_gate()
    }
}

#[derive(Debug, Clone, Default)]
/// Policy object that chooses a grooming duration from pet history first, then contracted breed/coat defaults.
pub struct EstimationPolicy;

impl EstimationPolicy {
    /// Estimates appointment minutes from source history or contract defaults and records any required review gate.
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

/// No-show and late-cancel boundary for protecting groomer capacity and rebooking policy.
pub mod no_show {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for rule in grooming workflows.
    pub enum Rule {
        /// Note history only grooming operational signal for schedule, estimate, history, or review handling.
        NoteHistoryOnly,
        /// Require deposit for rebooking grooming operational signal for schedule, estimate, history, or review handling.
        RequireDepositForRebooking,
        /// Manager review before rebooking grooming operational signal for schedule, estimate, history, or review handling.
        ManagerReviewBeforeRebooking,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Represents the count concept as a typed grooming operational contract instead of a raw primitive.
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
    /// Represents the late cancel count concept as a typed grooming operational contract instead of a raw primitive.
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
    /// Represents the history concept as a typed grooming operational contract instead of a raw primitive.
    pub struct History {
        /// Source-derived no shows carried by this grooming contract.
        pub no_shows: Count,
        /// Source-derived late cancels carried by this grooming contract.
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

        /// Returns the repeat behavior count evidence recorded on this grooming contract.
        pub const fn repeat_behavior_count(&self) -> u16 {
            self.no_shows.get().saturating_add(self.late_cancels.get())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for workflow outcomes in grooming workflows.
    pub enum Decision {
        /// Clear to rebook grooming operational signal for schedule, estimate, history, or review handling.
        ClearToRebook,
        /// Source-derived gate carried by this grooming contract.
        DepositRequired {
            /// Gate value carried by this review or workflow variant.
            gate: crate::policy::ReviewGate,
        },
        /// Source-derived gate carried by this grooming contract.
        ManagerReviewRequired {
            /// Gate value carried by this review or workflow variant.
            gate: crate::policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Represents the evaluation concept as a typed grooming operational contract instead of a raw primitive.
    pub struct Evaluation {
        /// Source-derived customer id carried by this grooming contract.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Source-derived history carried by this grooming contract.
        pub history: History,
    }

    #[derive(Debug, Clone)]
    /// Represents the policy concept as a typed grooming operational contract instead of a raw primitive.
    pub struct Policy {
        rule: Rule,
    }

    impl Policy {
        /// Assembles this grooming value from already-validated domain parts.
        pub const fn new(rule: Rule) -> Self {
            Self { rule }
        }

        /// Evaluates grooming source facts into a rebooking or review decision.
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
    /// Decision vocabulary for care reference in grooming workflows.
    pub enum CareReference {
        /// Sensitive skin product grooming operational signal for schedule, estimate, history, or review handling.
        SensitiveSkinProduct,
        /// Medicated product requires review grooming operational signal for schedule, estimate, history, or review handling.
        MedicatedProductRequiresReview,
        /// Handling or medical concern grooming operational signal for schedule, estimate, history, or review handling.
        HandlingOrMedicalConcern,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for service outcome in grooming workflows.
    pub enum ServiceOutcome {
        /// Completed grooming operational signal for schedule, estimate, history, or review handling.
        Completed,
        /// No show grooming operational signal for schedule, estimate, history, or review handling.
        NoShow,
        /// Late cancelled grooming operational signal for schedule, estimate, history, or review handling.
        LateCancelled,
        /// Needs follow up grooming operational signal for schedule, estimate, history, or review handling.
        NeedsFollowUp,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for approval state in grooming workflows.
    pub enum ApprovalState {
        /// Draft grooming operational signal for schedule, estimate, history, or review handling.
        Draft,
        /// Source-derived gate carried by this grooming contract.
        ReviewRequired {
            /// Gate value carried by this review or workflow variant.
            gate: crate::policy::ReviewGate,
        },
        /// Source-derived groomer id carried by this grooming contract.
        ApprovedByGroomer {
            /// Groomer id value carried by this review or workflow variant.
            groomer_id: StaffId,
        },
        /// Source-derived gate carried by this grooming contract.
        Rejected {
            /// Gate value carried by this review or workflow variant.
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
    /// Represents the service history entry concept as a typed grooming operational contract instead of a raw primitive.
    pub struct ServiceHistoryEntry {
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Source-derived location id carried by this grooming contract.
        pub location_id: LocationId,
        /// Requested service that drives scheduling and labor estimates.
        pub service: super::Service,
        /// Source-derived completed on carried by this grooming contract.
        pub completed_on: NaiveDate,
        /// Source-derived outcome carried by this grooming contract.
        pub outcome: ServiceOutcome,
        /// Source-derived approval carried by this grooming contract.
        pub approval: ApprovalState,
        #[builder(default)]
        style_notes: Vec<style_note::StyleNote>,
        #[builder(default)]
        care_refs: Vec<CareReference>,
        duration: Option<AppointmentMinutes>,
    }

    impl ServiceHistoryEntry {
        /// Returns the style notes evidence recorded on this grooming contract.
        pub fn style_notes(&self) -> &[style_note::StyleNote] {
            &self.style_notes
        }

        /// Returns the care refs evidence recorded on this grooming contract.
        pub fn care_refs(&self) -> &[CareReference] {
            &self.care_refs
        }

        /// Returns the duration evidence recorded on this grooming contract.
        pub const fn duration(&self) -> Option<AppointmentMinutes> {
            self.duration
        }

        /// Reports whether care-team review is needed before proceeding.
        pub const fn requires_review(&self) -> bool {
            self.approval.requires_review() || !self.care_refs.is_empty()
        }
    }
}

/// Rebooking cadence boundary for identifying due, overdue, or history-insufficient grooming follow-up.
pub mod rebooking {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Represents the cadence weeks concept as a typed grooming operational contract instead of a raw primitive.
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
    /// Decision vocabulary for cadence weeks error in grooming workflows.
    pub enum CadenceWeeksError {
        #[error("grooming cadence requires at least one week")]
        /// Zero weeks grooming operational signal for schedule, estimate, history, or review handling.
        ZeroWeeks,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Represents the ordinary cadence weeks concept as a typed grooming operational contract instead of a raw primitive.
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
    /// Decision vocabulary for ordinary cadence weeks error in grooming workflows.
    pub enum OrdinaryCadenceWeeksError {
        #[error("ordinary grooming rebooking cadence must be between 2 and 8 weeks")]
        /// Outside ordinary grooming band grooming operational signal for schedule, estimate, history, or review handling.
        OutsideOrdinaryGroomingBand,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for cadence in grooming workflows.
    pub enum Cadence {
        /// Every weeks grooming operational signal for schedule, estimate, history, or review handling.
        EveryWeeks(CadenceWeeks),
        /// As needed grooming operational signal for schedule, estimate, history, or review handling.
        AsNeeded,
        /// Groomer recommended grooming operational signal for schedule, estimate, history, or review handling.
        GroomerRecommended,
        /// Provider role or status could not be mapped confidently.
        Unknown,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized reservation states observed during source-data ingestion.
    pub enum Status {
        /// Due later grooming operational signal for schedule, estimate, history, or review handling.
        DueLater,
        /// Due now grooming operational signal for schedule, estimate, history, or review handling.
        DueNow,
        /// Overdue grooming operational signal for schedule, estimate, history, or review handling.
        Overdue,
        /// Needs groomer recommendation grooming operational signal for schedule, estimate, history, or review handling.
        NeedsGroomerRecommendation,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for rationale in grooming workflows.
    pub enum Rationale {
        /// Last completed service cadence grooming operational signal for schedule, estimate, history, or review handling.
        LastCompletedServiceCadence,
        /// No completed history grooming operational signal for schedule, estimate, history, or review handling.
        NoCompletedHistory,
        /// Groomer recommended cadence required grooming operational signal for schedule, estimate, history, or review handling.
        GroomerRecommendedCadenceRequired,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Represents the recommendation concept as a typed grooming operational contract instead of a raw primitive.
    pub struct Recommendation {
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Source-derived due on carried by this grooming contract.
        pub due_on: Option<NaiveDate>,
        /// Source-derived status carried by this grooming contract.
        pub status: Status,
        /// Source-derived rationale carried by this grooming contract.
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Default)]
    /// Represents the policy concept as a typed grooming operational contract instead of a raw primitive.
    pub struct Policy;

    impl Policy {
        /// Returns the recommend from history evidence recorded on this grooming contract.
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

/// Reminder boundary for drafting appointment confirmations, prep instructions, and cadence winback messages.
pub mod reminder {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for rule in grooming workflows.
    pub enum Rule {
        /// One week before grooming operational signal for schedule, estimate, history, or review handling.
        OneWeekBefore,
        /// Forty eight hours before grooming operational signal for schedule, estimate, history, or review handling.
        FortyEightHoursBefore,
        /// Morning of grooming operational signal for schedule, estimate, history, or review handling.
        MorningOf,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for kind in grooming workflows.
    pub enum Kind {
        /// Appointment confirmation grooming operational signal for schedule, estimate, history, or review handling.
        AppointmentConfirmation,
        /// Prep instructions grooming operational signal for schedule, estimate, history, or review handling.
        PrepInstructions,
        /// Morning of grooming operational signal for schedule, estimate, history, or review handling.
        MorningOf,
        /// Rebooking due grooming operational signal for schedule, estimate, history, or review handling.
        RebookingDue,
        /// Lapsed cadence winback grooming operational signal for schedule, estimate, history, or review handling.
        LapsedCadenceWinback,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for consent in grooming workflows.
    pub enum Consent {
        /// Granted grooming operational signal for schedule, estimate, history, or review handling.
        Granted,
        /// Not granted grooming operational signal for schedule, estimate, history, or review handling.
        NotGranted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Decision vocabulary for send boundary in grooming workflows.
    pub enum SendBoundary {
        /// Draft requires approval grooming operational signal for schedule, estimate, history, or review handling.
        DraftRequiresApproval,
        /// Ready for approved send grooming operational signal for schedule, estimate, history, or review handling.
        ReadyForApprovedSend,
        /// Suppressed until consent grooming operational signal for schedule, estimate, history, or review handling.
        SuppressedUntilConsent,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Represents the plan concept as a typed grooming operational contract instead of a raw primitive.
    pub struct Plan {
        /// Source-derived customer id carried by this grooming contract.
        pub customer_id: CustomerId,
        /// Source-derived kind carried by this grooming contract.
        pub kind: Kind,
        boundary: SendBoundary,
    }

    impl Plan {
        /// Returns the send boundary evidence recorded on this grooming contract.
        pub const fn send_boundary(&self) -> SendBoundary {
            self.boundary
        }

        /// Returns the customer message review gate recorded on this grooming contract.
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
    /// Represents the policy concept as a typed grooming operational contract instead of a raw primitive.
    pub struct Policy;

    impl Policy {
        /// Builds a grooming reminder plan from customer consent and reminder purpose.
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
/// Location grooming contract tying calendar assignment, estimate policy, no-show rules, rebooking cadence, reminders, and history retention together.
pub struct Contract {
    /// Source-derived calendar carried by this grooming contract.
    pub calendar: calendar::Policy,
    #[builder(default)]
    /// Source-derived time estimates carried by this grooming contract.
    pub time_estimates: Vec<breed_coat::TimeEstimate>,
    /// Source-derived no show carried by this grooming contract.
    pub no_show: no_show::Rule,
    /// Source-derived rebooking carried by this grooming contract.
    pub rebooking: rebooking::Cadence,
    #[builder(default)]
    /// Source-derived reminders carried by this grooming contract.
    pub reminders: Vec<reminder::Rule>,
    /// Source-derived history carried by this grooming contract.
    pub history: HistoryRequirement,
}

impl Contract {
    /// Reports whether prior no-shows should trigger a deposit or manager review before rebooking.
    pub fn requires_deposit_after_no_show(&self) -> bool {
        matches!(
            self.no_show,
            no_show::Rule::RequireDepositForRebooking | no_show::Rule::ManagerReviewBeforeRebooking
        )
    }
    /// Builds a representative PetSuites-style grooming contract for docs/tests without claiming it is live policy.
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
