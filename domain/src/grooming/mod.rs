//! Grooming service-line rules for duration estimation and evidence-only no-show, cadence, and reminder review.
//!
//! Operators use this module to estimate groomer time and inspect reported history. Serializable rebooking and reminder values remain suppressed evidence: they cannot create a candidate, queue, task, plan, slot proposal, review packet, or customer draft.
//!
//! Use it for duration and evidence classification, not follow-up preparation. Human review cannot mint the unavailable opaque, non-serializable rebooking eligibility authority.
//!
//! The authoritative facts are the location rules, the requested `Service`, breed/coat facts on `EstimationRequest`, prior approved `history::ServiceHistoryEntry` records, pet/customer/location/staff identity from `domain::entities`, and shared `domain::policy::ReviewGate` approvals. Provider catalog names, adapter defaults, and AI suggestions must be promoted into these values or remain pending review evidence.
//!
//! This module must not book or move appointments, assign a live provider-calendar slot, send a customer message, charge or waive a deposit, or decide medical/handling safety. `reminder::Plan::customer_message_gate` returns no gate because current reminder evidence cannot become a draft.

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

impl ReviewRequirement {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Grooming duration decision with evidence, confidence, and the review gate needed before calendar use.
pub struct DurationEstimate {
    minutes: AppointmentMinutes,
    basis: EstimateBasis,
    confidence: EstimateConfidence,
    review: ReviewRequirement,
}

impl DurationEstimate {
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
}

#[derive(Debug, Clone, Default)]
/// Policy object that chooses a grooming duration from pet history first, then location breed/coat defaults.
pub struct EstimationPolicy;

impl EstimationPolicy {}

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
        /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Late-cancel count considered with no-shows during grooming rebooking review.
    pub struct LateCancelCount(u16);

    impl LateCancelCount {
        /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Caller-constructible repeat grooming history retained as evidence only.
    pub struct History {
        /// No shows from source or staff evidence used during grooming schedule/rebooking review; it does not authorize live changes by itself.
        pub no_shows: Count,
        /// Late cancels from source or staff evidence used during grooming schedule/rebooking review; it does not authorize live changes by itself.
        pub late_cancels: LateCancelCount,
    }

    impl History {
        /// Returns the repeat behavior count value used by grooming schedule/rebooking review.
        pub const fn repeat_behavior_count(&self) -> u16 {
            self.no_shows.get().saturating_add(self.late_cancels.get())
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Grooming rebooking evaluation packet tying customer, pet, and repeat-history facts together.
    pub struct Evaluation {
        /// Customer whose grooming reminder, deposit review, or rebooking packet is being prepared.
        pub customer_id: CustomerId,
        /// Pet receiving the grooming or care service.
        pub pet_id: PetId,
        /// No-show and late-cancel history staff review before choosing a rebooking path.
        pub history: History,
    }

    impl std::fmt::Debug for Evaluation {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("Evaluation([REDACTED])")
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
    /// Caller-reported service outcome retained for history and estimates.
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
        /// Caller-reported service outcome used for history and estimate context, not follow-up authority.
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
    }
}

/// Rebooking cadence policy for identifying due, overdue, or history-insufficient grooming follow-up.
pub mod rebooking {
    use core::num::NonZeroU8;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Reported grooming cadence in weeks; it does not establish rebooking eligibility.
    pub struct CadenceWeeks(NonZeroU8);

    impl CadenceWeeks {
        /// Rejects zero or unsupported grooming values before they affect groomer calendars, duration estimates, deposits, reminders, or rebooking prompts.
        pub const fn try_new(value: u8) -> std::result::Result<Self, CadenceWeeksError> {
            match NonZeroU8::new(value) {
                Some(value) => Ok(Self(value)),
                None => Err(CadenceWeeksError::ZeroWeeks),
            }
        }

        /// Promotes an already-proven positive week count without repeating validation.
        pub const fn from_nonzero(value: NonZeroU8) -> Self {
            Self(value)
        }

        /// Preserves the proof that this cadence is positive across trusted adapters.
        pub const fn into_nonzero(self) -> NonZeroU8 {
            self.0
        }

        /// Returns the grooming number used by scheduling, estimate, reminder, or rebooking calculations.
        pub const fn get(self) -> u8 {
            self.0.get()
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
    /// Reported cadence source retained as non-authoritative evidence.
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
    /// Caller-constructible rationale label for reported grooming cadence evidence.
    pub enum Rationale {
        /// Staff can see the last completed service cadence grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        LastCompletedServiceCadence,
        /// Staff can see the no completed history grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        NoCompletedHistory,
        /// Staff can see the groomer recommended cadence required grooming state during grooming scheduling, estimate, history, rebooking, reminder, or review work.
        GroomerRecommendedCadenceRequired,
    }
}

/// Grooming reminder timing rules owned by location service contracts.
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
    /// Reported rebooking cadence retained as suppressed evidence.
    pub rebooking: rebooking::Cadence,
    #[builder(default)]
    /// Compatibility reminder timing labels; current runtime cannot create drafts from them.
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
}
