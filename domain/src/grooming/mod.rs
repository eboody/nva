//! Grooming service-line rules for pet-resort scheduling, no-show, rebooking, reminder, and review queues.
//!
//! Operators use this module to answer grooming queue questions without rereading notes by hand: how much groomer time a mini/full groom, bath, nail service, or coat/skin add-on should reserve; whether repeat no-show history requires a deposit or manager review; when a completed service should become a rebooking prompt; and whether a reminder draft is safe to prepare. The labor reduction is triage and evidence assembly, not unattended execution.
//!
//! Use it when the business question is "what grooming work can be prepared for staff or customer review, and what calendar, deposit, handling, or message approval still blocks live execution?" Next step: start with the location rules and `Service` for policy and request type, then follow `duration_estimate`, `no_show`, `rebooking`, `reminder`, or `calendar` depending on the queue you are trying to explain.
//!
//! The authoritative facts are the location rules, the requested `Service`, breed/coat facts on `EstimationRequest`, prior approved `history::ServiceHistoryEntry` records, pet/customer/location/staff identity from `domain::entities`, and shared `domain::policy::ReviewGate` approvals. Provider catalog names, adapter defaults, and AI suggestions must be promoted into these values or remain pending review evidence.
//!
//! This module must not book or move appointments, assign a live provider-calendar slot, send a customer message, charge or waive a deposit, or decide medical/handling safety on its own. `ReviewRequirement::calendar_execution_gate`, `no_show::Decision`, and `reminder::Plan::customer_message_gate` preserve the human review gates that protect pets, customers, groomers, and managers before app/storage/integration layers perform live work.

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
            /// Rejects zero or unsupported grooming values before they affect groomer calendars, duration estimates, deposits, reminders, or rebooking prompts.
            pub const fn try_new(value: $primitive) -> std::result::Result<Self, $error> {
                if value == 0 {
                    return Err($error::Zero);
                }
                Ok(Self(value))
            }

            /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
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

/// Groomer-calendar policy for assigning grooming work without inventing availability.
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
/// Breed/coat inputs for converting pet profile facts into labor-time estimates.
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
        /// Creates this grooming value from already-checked resort workflow inputs.
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

        /// Returns the minutes value used by grooming schedule/rebooking review.
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
/// Grooming estimate request assembled from pet profile facts before staff propose any calendar change.
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
    /// Estimate came from the location breed/coat policy.
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

    /// Returns the minutes value used by grooming schedule/rebooking review.
    pub const fn minutes(&self) -> AppointmentMinutes {
        self.minutes
    }

    /// Returns the basis value used by grooming schedule/rebooking review.
    pub const fn basis(&self) -> EstimateBasis {
        self.basis
    }

    /// Returns the confidence value used by grooming schedule/rebooking review.
    pub const fn confidence(&self) -> EstimateConfidence {
        self.confidence
    }

    /// Returns the review value used by grooming schedule/rebooking review.
    pub const fn review(&self) -> ReviewRequirement {
        self.review
    }

    /// Maps the grooming review lane to the workflow gate that must approve scheduling.
    pub const fn calendar_execution_gate(&self) -> Option<crate::policy::ReviewGate> {
        self.review.calendar_execution_gate()
    }
}

#[derive(Debug, Clone, Default)]
/// Policy object that chooses a grooming duration from pet history first, then location breed/coat defaults.
pub struct EstimationPolicy;

impl EstimationPolicy {
    /// Estimates appointment minutes from source history or local policy defaults and records any required review gate.
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

/// No-show and late-cancel policy for protecting groomer capacity and rebooking decisions.
pub mod no_show {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Rebooking rule that tells staff whether history only, deposit review, or manager review applies.
    pub enum Rule {
        /// Staff can see the note history only grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        NoteHistoryOnly,
        /// Staff can see the require deposit for rebooking grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        RequireDepositForRebooking,
        /// Staff can see the manager review before rebooking grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        ManagerReviewBeforeRebooking,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// No-show count considered during grooming deposit and rebooking review.
    pub struct Count(u16);

    impl Count {
        /// Rejects zero or unsupported grooming values before they affect groomer calendars, duration estimates, deposits, reminders, or rebooking prompts.
        pub const fn try_new(value: u16) -> std::result::Result<Self, std::convert::Infallible> {
            Ok(Self(value))
        }

        /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Late-cancel count considered with no-shows during grooming rebooking review.
    pub struct LateCancelCount(u16);

    impl LateCancelCount {
        /// Rejects zero or unsupported grooming values before they affect groomer calendars, duration estimates, deposits, reminders, or rebooking prompts.
        pub const fn try_new(value: u16) -> std::result::Result<Self, std::convert::Infallible> {
            Ok(Self(value))
        }

        /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Repeat grooming history staff review before clearing a rebooking path.
    pub struct History {
        /// No shows from source or staff evidence used during grooming schedule/rebooking review; it does not authorize live changes by itself.
        pub no_shows: Count,
        /// Late cancels from source or staff evidence used during grooming schedule/rebooking review; it does not authorize live changes by itself.
        pub late_cancels: LateCancelCount,
    }

    impl History {
        /// Creates this grooming value from already-checked resort workflow inputs.
        pub const fn new(no_shows: Count, late_cancels: LateCancelCount) -> Self {
            Self {
                no_shows,
                late_cancels,
            }
        }

        /// Returns the repeat behavior count value used by grooming schedule/rebooking review.
        pub const fn repeat_behavior_count(&self) -> u16 {
            self.no_shows.get().saturating_add(self.late_cancels.get())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Grooming rebooking outcome that tells staff whether to clear, collect a deposit, or seek manager review.
    pub enum Decision {
        /// Staff can see the clear to rebook grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        ClearToRebook,
        /// Review gate that must clear before this grooming decision can trigger a live schedule, deposit, or message action.
        DepositRequired {
            /// Approval gate staff must clear before acting on this variant.
            gate: crate::policy::ReviewGate,
        },
        /// Review gate that must clear before this grooming decision can trigger a live schedule, deposit, or message action.
        ManagerReviewRequired {
            /// Approval gate staff must clear before acting on this variant.
            gate: crate::policy::ReviewGate,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Grooming rebooking evaluation packet tying customer, pet, and repeat-history facts together.
    pub struct Evaluation {
        /// Customer whose grooming reminder, deposit review, or rebooking packet is being prepared.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// No-show and late-cancel history staff review before choosing a rebooking path.
        pub history: History,
    }

    #[derive(Debug, Clone)]
    /// Grooming policy object that turns local rules into staff review decisions.
    pub struct Policy {
        rule: Rule,
    }

    impl Policy {
        /// Creates this grooming value from already-checked resort workflow inputs.
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

/// History workflow gate for the grooming schedule, estimate, history, rebooking, reminder, or review workflow.
pub mod history {
    use super::*;

    /// Style note workflow gate for the grooming schedule, estimate, history, rebooking, reminder, or review workflow.
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
    /// Care references that keep product, medical, or handling notes visible to groomer review.
    pub enum CareReference {
        /// Staff can see the sensitive skin product grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        SensitiveSkinProduct,
        /// Staff can see the medicated product requires review grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        MedicatedProductRequiresReview,
        /// Staff can see the handling or medical concern grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        HandlingOrMedicalConcern,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Service outcome recorded for grooming history, estimates, and rebooking prompts.
    pub enum ServiceOutcome {
        /// Staff can see the completed grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        Completed,
        /// Staff can see the no show grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        NoShow,
        /// Staff can see the late cancelled grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        LateCancelled,
        /// Staff can see the needs follow up grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        NeedsFollowUp,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Approval state controlling whether grooming history can support future estimates or rebooking.
    pub enum ApprovalState {
        /// Staff can see the draft grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        Draft,
        /// Review gate that must clear before this grooming decision can trigger a live schedule, deposit, or message action.
        ReviewRequired {
            /// Approval gate staff must clear before acting on this variant.
            gate: crate::policy::ReviewGate,
        },
        /// Groomer who approved the history entry for later estimate, care, or rebooking review.
        ApprovedByGroomer {
            /// Groomer who approved this grooming history or review state.
            groomer_id: StaffId,
        },
        /// Review gate that must clear before this grooming decision can trigger a live schedule, deposit, or message action.
        Rejected {
            /// Approval gate staff must clear before acting on this variant.
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
    /// Grooming history entry used for future duration estimates, style continuity, care review, and rebooking.
    pub struct ServiceHistoryEntry {
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Resort location whose grooming history should be considered for this pet.
        pub location_id: LocationId,
        /// Requested service that drives scheduling and labor estimates.
        pub service: super::Service,
        /// Date the grooming outcome was completed or recorded for cadence and history review.
        pub completed_on: NaiveDate,
        /// Service outcome used to decide whether future estimates, rebooking prompts, or follow-up are appropriate.
        pub outcome: ServiceOutcome,
        /// Approval state that keeps sensitive grooming history out of automation until review clears.
        pub approval: ApprovalState,
        #[builder(default)]
        style_notes: Vec<style_note::StyleNote>,
        #[builder(default)]
        care_refs: Vec<CareReference>,
        duration: Option<AppointmentMinutes>,
    }

    impl ServiceHistoryEntry {
        /// Returns the style notes value used by grooming schedule/rebooking review.
        pub fn style_notes(&self) -> &[style_note::StyleNote] {
            &self.style_notes
        }

        /// Returns the care refs value used by grooming schedule/rebooking review.
        pub fn care_refs(&self) -> &[CareReference] {
            &self.care_refs
        }

        /// Returns the duration value used by grooming schedule/rebooking review.
        pub const fn duration(&self) -> Option<AppointmentMinutes> {
            self.duration
        }

        /// Reports whether care-team review is needed before proceeding.
        pub const fn requires_review(&self) -> bool {
            self.approval.requires_review() || !self.care_refs.is_empty()
        }
    }
}

/// Rebooking cadence policy for identifying due, overdue, or history-insufficient grooming follow-up.
pub mod rebooking {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Grooming cadence in weeks for due/overdue rebooking prompts.
    pub struct CadenceWeeks(u8);

    impl CadenceWeeks {
        /// Rejects zero or unsupported grooming values before they affect groomer calendars, duration estimates, deposits, reminders, or rebooking prompts.
        pub const fn try_new(value: u8) -> std::result::Result<Self, CadenceWeeksError> {
            if value == 0 {
                return Err(CadenceWeeksError::ZeroWeeks);
            }
            Ok(Self(value))
        }

        /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
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
    /// Cadence validation error for rebooking prompts that cannot use zero weeks.
    pub enum CadenceWeeksError {
        #[error("grooming cadence requires at least one week")]
        /// Staff can see the zero weeks grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        ZeroWeeks,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Ordinary grooming cadence band used when staff expect repeat appointments every few weeks.
    pub struct OrdinaryCadenceWeeks(u8);

    impl OrdinaryCadenceWeeks {
        /// Rejects zero or unsupported grooming values before they affect groomer calendars, duration estimates, deposits, reminders, or rebooking prompts.
        pub const fn try_new(value: u8) -> std::result::Result<Self, OrdinaryCadenceWeeksError> {
            if value < 2 || value > 8 {
                return Err(OrdinaryCadenceWeeksError::OutsideOrdinaryGroomingBand);
            }
            Ok(Self(value))
        }

        /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
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
    /// Cadence validation error for values outside the ordinary grooming rebooking band.
    pub enum OrdinaryCadenceWeeksError {
        #[error("ordinary grooming rebooking cadence must be between 2 and 8 weeks")]
        /// Staff can see the outside ordinary grooming band grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        OutsideOrdinaryGroomingBand,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Rebooking cadence source used for due-date prompts and groomer-recommended follow-up.
    pub enum Cadence {
        /// Staff can see the every weeks grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        EveryWeeks(CadenceWeeks),
        /// Staff can see the as needed grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        AsNeeded,
        /// Staff can see the groomer recommended grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        GroomerRecommended,
        /// Provider role or status could not be mapped confidently.
        Unknown,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Normalized reservation states observed during source-data ingestion.
    pub enum Status {
        /// Staff can see the due later grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        DueLater,
        /// Staff can see the due now grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        DueNow,
        /// Staff can see the overdue grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        Overdue,
        /// Staff can see the needs groomer recommendation grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        NeedsGroomerRecommendation,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Reason explaining why a grooming rebooking prompt is due or needs groomer input.
    pub enum Rationale {
        /// Staff can see the last completed service cadence grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        LastCompletedServiceCadence,
        /// Staff can see the no completed history grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        NoCompletedHistory,
        /// Staff can see the groomer recommended cadence required grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        GroomerRecommendedCadenceRequired,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Grooming rebooking recommendation staff can review before drafting customer follow-up.
    pub struct Recommendation {
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// Date when the next grooming reminder or rebooking prompt becomes due.
        pub due_on: Option<NaiveDate>,
        /// Rebooking status staff use to decide whether to prompt, wait, or request groomer input.
        pub status: Status,
        /// Reason explaining why the rebooking recommendation is due, overdue, or blocked for groomer input.
        pub rationale: Rationale,
    }

    #[derive(Debug, Clone, Default)]
    /// Grooming policy object that turns local rules into staff review decisions.
    pub struct Policy;

    impl Policy {
        /// Returns the recommend from history value used by grooming schedule/rebooking review.
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

/// Reminder policy for drafting appointment confirmations, prep instructions, and cadence winback messages.
pub mod reminder {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Rebooking rule that tells staff whether history only, deposit review, or manager review applies.
    pub enum Rule {
        /// Staff can see the one week before grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        OneWeekBefore,
        /// Staff can see the forty eight hours before grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        FortyEightHoursBefore,
        /// Staff can see the morning of grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        MorningOf,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Reminder purpose for grooming confirmations, prep instructions, and cadence follow-up drafts.
    pub enum Kind {
        /// Staff can see the appointment confirmation grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        AppointmentConfirmation,
        /// Staff can see the prep instructions grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        PrepInstructions,
        /// Staff can see the morning of grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        MorningOf,
        /// Staff can see the rebooking due grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        RebookingDue,
        /// Staff can see the lapsed cadence winback grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        LapsedCadenceWinback,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Customer-message consent state used before grooming reminder drafts proceed.
    pub enum Consent {
        /// Staff can see the granted grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        Granted,
        /// Staff can see the not granted grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        NotGranted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Customer-message send status for grooming reminder plans.
    pub enum SendBoundary {
        /// Staff can see the draft requires approval grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        DraftRequiresApproval,
        /// Staff can see the ready for approved send grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        ReadyForApprovedSend,
        /// Staff can see the suppressed until consent grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        SuppressedUntilConsent,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Grooming reminder plan that separates message purpose from send approval.
    pub struct Plan {
        /// Customer whose grooming reminder, deposit review, or rebooking packet is being prepared.
        pub customer_id: CustomerId,
        /// Reminder purpose that controls whether the draft is confirmation, prep, same-day, or cadence follow-up copy.
        pub kind: Kind,
        boundary: SendBoundary,
    }

    impl Plan {
        /// Returns the customer-message send gate value used by grooming schedule/rebooking review.
        pub const fn send_boundary(&self) -> SendBoundary {
            self.boundary
        }

        /// Returns the customer-message approval gate required before this grooming reminder is sent.
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
    /// Grooming policy object that turns local rules into staff review decisions.
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
/// Location grooming ruleset tying calendar assignment, estimate policy, no-show rules, rebooking cadence, reminders, and history retention together.
pub struct Contract {
    /// Calendar-assignment rule staff honor before drafting or reviewing grooming work.
    pub calendar: calendar::Policy,
    #[builder(default)]
    /// Breed/coat duration estimates staff use for groomer-calendar planning.
    pub time_estimates: Vec<breed_coat::TimeEstimate>,
    /// No-show rule that controls whether repeat history creates deposit or manager review.
    pub no_show: no_show::Rule,
    /// Rebooking cadence staff use when preparing due or overdue grooming prompts.
    pub rebooking: rebooking::Cadence,
    #[builder(default)]
    /// Reminder timing options that can be drafted only through customer-message review.
    pub reminders: Vec<reminder::Rule>,
    /// No-show and late-cancel history staff review before choosing a rebooking path.
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
    /// Builds representative PetSuites-style grooming rules for docs/tests without claiming they are live policy.
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
