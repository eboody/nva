//! Strategic AI-operations domain contracts for NVA Pet Resorts.
//!
//! This module fills the modeling gaps behind the strategic AI-ops roadmap: real-time lead
//! response, customer intelligence, capacity/labor optimization, permissioned knowledge-assistant
//! context, site financial insights, and generalized outcome attribution. All values remain
//! source-backed and review-gated; constructing these packets does not authorize customer sends,
//! schedule changes, provider/PMS writes, discounts, payments, or staffing mandates.

use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{entities, policy};

/// Source-system vocabulary for strategic AI operations beyond the first Gingr/local proof.
pub mod source {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Enterprise source system that can provide evidence for AI-ops recommendations.
    pub enum System {
        /// Telephony or call-center system carrying missed calls, voicemails, and call outcomes.
        Telephony,
        /// SMS provider carrying inbound/outbound text evidence and opt-out events.
        SmsProvider,
        /// Email inbox or transactional-email provider.
        Email,
        /// Web chat or website-assistant transcript source.
        WebChat,
        /// Website lead/intake form source.
        WebsiteForms,
        /// Marketing automation or campaign platform.
        MarketingAutomation,
        /// CRM/customer profile source.
        Crm,
        /// Finance/accounting source for site revenue, discounts, refunds, and costs.
        FinanceAccounting,
        /// HRIS, scheduling, or timekeeping source for labor evidence.
        WorkforceManagement,
        /// Document/SOP/vendor knowledge base.
        KnowledgeBase,
        /// Existing pet-resort operating system or PMS evidence.
        ProviderOrPms,
        /// Curated manual import with source-review responsibility.
        ManualImport,
    }
}

/// Time primitives used by optimization, financial, and lead-response models.
pub mod time {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Half-open UTC time window used for SLA, staffing, demand, and reporting buckets.
    pub struct Window {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    }

    impl Window {
        /// Creates a time window only when the end follows the start.
        pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, Error> {
            if end <= start {
                return Err(Error::EndMustFollowStart);
            }
            Ok(Self { start, end })
        }

        /// Start instant of the window.
        pub const fn start(&self) -> DateTime<Utc> {
            self.start
        }

        /// End instant of the window.
        pub const fn end(&self) -> DateTime<Utc> {
            self.end
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Time-window validation failures.
    pub enum Error {
        #[error("time window end must follow start")]
        /// The end timestamp did not follow the start timestamp.
        EndMustFollowStart,
    }
}

/// Shared access, role, visibility, and allowed-use contracts.
pub mod access {
    use super::*;

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
    /// Stable actor id for a staff member, role account, or approved operator context.
    pub struct ActorId(String);

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
    /// Human title used when scoping assistant context and data access.
    pub struct Title(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    /// Role used to scope operational context and permitted assistant retrieval.
    pub enum ActorRole {
        /// Front-desk staff handling lead, booking, checkout, and customer work.
        FrontDesk,
        /// Care/kennel team member handling pet care and facility tasks.
        CareStaff,
        /// Groomer role.
        Groomer,
        /// Trainer role.
        Trainer,
        /// Site manager or general manager.
        SiteManager,
        /// Regional operations role.
        RegionalOperations,
        /// Finance analyst or regional/site financial reviewer.
        Finance,
        /// Marketing role with restricted access to operations-only facts.
        Marketing,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Visibility boundary for customer, pet, financial, and knowledge context.
    pub enum VisibilityScope {
        /// Only operational staff with local/site context may use the fact.
        OperationsOnly,
        /// Marketing may use the fact after consent and segmentation review.
        MarketingEligible,
        /// Site managers may use the fact for local operations.
        SiteManagement,
        /// Regional users may use aggregated or approved cross-site facts.
        RegionalAggregate,
        /// Sensitive fact requires a specific review gate before use.
        ReviewGated,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Purpose for which a fact may be used.
    pub enum AllowedUse {
        /// Use to personalize service or staff handling.
        ServicePersonalization,
        /// Use for internal decision support only.
        InternalDecisionSupport,
        /// Use in a marketing campaign after consent and scope review.
        MarketingCampaign,
        /// Use for financial or performance reporting.
        FinancialAnalysis,
        /// Use for labor or staffing recommendation review.
        LaborOptimization,
    }
}

/// Identity matching contracts for customer, pet, and household reconciliation.
pub mod identity {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Match confidence band for source-to-domain identity promotion.
    pub enum Confidence {
        /// Low confidence; human review should be visible.
        Low,
        /// Medium confidence; usable only with caveats or review.
        Medium,
        /// High confidence from stable source evidence.
        High,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Customer identity resolution result for lead, CRM, and retention workflows.
    pub enum Match {
        /// No matching customer was found.
        None,
        /// One candidate customer exists with explicit confidence.
        Candidate {
            /// Candidate customer id.
            customer_id: entities::CustomerId,
            /// Confidence in the match.
            confidence: Confidence,
        },
        /// Multiple plausible customers require staff review.
        Ambiguous {
            /// Candidate customer ids.
            candidates: Vec<entities::CustomerId>,
        },
    }
}

/// Communication consent and purpose contracts shared by leads, retention, and CRM.
pub mod communication {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Channel used for communication evidence or draft outreach.
    pub enum Channel {
        /// Phone call or callback.
        Phone,
        /// SMS/text.
        Sms,
        /// Email.
        Email,
        /// Customer portal.
        Portal,
        /// Internal staff-only note/task.
        Internal,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Consent purpose that determines whether a channel can be used.
    pub enum Purpose {
        /// Transactional response to a fresh lead or inquiry.
        TransactionalLeadResponse,
        /// Marketing or winback outreach.
        MarketingRetention,
        /// Internal service personalization only.
        InternalServiceContext,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Consent state for a channel/purpose pair.
    pub enum ConsentStatus {
        /// Source evidence grants the use.
        Granted,
        /// Consent evidence is missing.
        Missing,
        /// Customer opted out or suppressed the channel/purpose.
        OptedOut,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Source-backed consent evidence for one channel and purpose.
    pub struct ConsentEvidence {
        channel: Channel,
        purpose: Purpose,
        status: ConsentStatus,
        source: source::System,
    }

    impl ConsentEvidence {
        /// Returns whether this consent evidence allows use.
        pub const fn is_granted(&self) -> bool {
            matches!(self.status, ConsentStatus::Granted)
        }

        /// Returns whether this evidence grants the exact channel/purpose pair.
        pub fn permits(&self, channel: Channel, purpose: Purpose) -> bool {
            self.channel == channel && self.purpose == purpose && self.is_granted()
        }

        /// Channel covered by this evidence.
        pub const fn channel(&self) -> Channel {
            self.channel
        }
    }
}

/// Real-time lead response contracts.
pub mod lead_response {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    /// Source event id for a lead-response event.
    pub struct EventId(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 240),
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
    /// Idempotency key used to deduplicate replayed lead events.
    pub struct IdempotencyKey(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    /// Campaign or source attribution label.
    pub struct Campaign(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Positive minute count for SLA and labor calculations.
    pub struct Minutes(u16);

    impl Minutes {
        /// Creates a nonzero minute count.
        pub const fn try_new(value: u16) -> Result<Self, Error> {
            if value == 0 {
                return Err(Error::ZeroMinutes);
            }
            Ok(Self(value))
        }

        /// Raw minute count.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Kind of realtime lead event.
    pub enum EventKind {
        /// Missed phone call.
        MissedCall,
        /// Voicemail left by a prospect/customer.
        Voicemail,
        /// Web form submitted.
        WebsiteFormSubmitted,
        /// SMS inbound.
        SmsInbound,
        /// Web chat inbound.
        WebChatInbound,
        /// Abandoned booking or quote flow.
        AbandonedBookingFlow,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Source-backed lead event ready for dedupe, SLA, and safe response review.
    pub struct Event {
        id: EventId,
        idempotency_key: IdempotencyKey,
        location_id: entities::LocationId,
        kind: EventKind,
        received_at: DateTime<Utc>,
        source_system: source::System,
        customer_match: identity::Match,
        service_intent: entities::ServiceKind,
    }

    impl Event {
        /// Source system that supplied this lead event.
        pub const fn source_system(&self) -> source::System {
            self.source_system
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// SLA target for lead response.
    pub enum SlaTarget {
        /// First response should happen within this number of minutes.
        FirstResponseWithinMinutes(Minutes),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Current SLA state.
    pub enum SlaStatus {
        /// SLA is open.
        Open,
        /// Response satisfied the SLA.
        Met,
        /// SLA was breached.
        Breached,
        /// After-hours policy paused the response clock.
        PausedAfterHours,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Lead-response SLA evidence.
    pub struct ResponseSla {
        target: SlaTarget,
        received_at: DateTime<Utc>,
        due_at: DateTime<Utc>,
        status: SlaStatus,
    }

    impl ResponseSla {
        /// Returns whether an attempted first response occurred before the due time.
        pub fn is_met_at(&self, at: DateTime<Utc>) -> bool {
            matches!(self.status, SlaStatus::Open | SlaStatus::Met)
                && at >= self.received_at
                && at <= self.due_at
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Outcome of a contact attempt.
    pub enum AttemptOutcome {
        /// Draft was prepared and waits on review.
        DraftedForReview,
        /// Staff reached the customer.
        ReachedCustomer,
        /// Voicemail left.
        VoicemailLeft,
        /// No answer.
        NoAnswer,
        /// Bounced/failed channel evidence.
        Failed,
        /// Customer opted out.
        OptedOut,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// One response attempt for a lead.
    pub struct ContactAttempt {
        attempted_at: DateTime<Utc>,
        channel: communication::Channel,
        purpose: communication::Purpose,
        outcome: AttemptOutcome,
        review_gate: policy::ReviewGate,
    }

    impl ContactAttempt {
        /// Review gate tied to this attempt.
        pub fn review_gate(&self) -> policy::ReviewGate {
            self.review_gate.clone()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Attribution source for a converted lead.
    pub enum AttributionSource {
        /// Missed-call recovery.
        MissedCall,
        /// Website form.
        WebsiteForm,
        /// SMS conversation.
        Sms,
        /// Marketing campaign.
        Campaign,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Conversion attribution retained for lead-response value proof.
    pub struct ConversionAttribution {
        source: AttributionSource,
        campaign: Option<Campaign>,
        converted_reservation_id: Option<entities::reservation::Id>,
        estimated_value_cents: Option<financial::MoneyCents>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Reviewable lead response packet.
    pub struct ResponsePacket {
        event: Event,
        sla: ResponseSla,
        consent: communication::ConsentEvidence,
        attempts: Vec<ContactAttempt>,
        attribution: ConversionAttribution,
    }

    impl ResponsePacket {
        /// Lead event in the packet.
        pub const fn event(&self) -> &Event {
            &self.event
        }

        /// Returns whether first response satisfies the SLA at the supplied attempt time.
        pub fn first_response_sla_is_met_at(&self, at: DateTime<Utc>) -> bool {
            self.sla.is_met_at(at)
        }

        /// Returns whether any attempt remains behind a customer-message approval gate.
        pub fn requires_customer_message_approval(&self) -> bool {
            self.attempts
                .iter()
                .any(|attempt| attempt.review_gate() == policy::ReviewGate::CustomerMessageApproval)
        }

        /// Returns whether packet consent grants the exact outbound response channel and purpose.
        pub fn consent_allows_response(
            &self,
            channel: communication::Channel,
            purpose: communication::Purpose,
        ) -> bool {
            self.consent.permits(channel, purpose)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Lead-response validation error.
    pub enum Error {
        #[error("minutes must be greater than zero")]
        /// Zero minutes would erase the SLA meaning.
        ZeroMinutes,
    }
}

/// CRM/customer-intelligence contracts.
pub mod crm {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    /// Structured CRM note id.
    pub struct NoteId(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 2000),
        derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
    )]
    /// Structured note body after staff/source review.
    pub struct NoteBody(String);

    impl fmt::Debug for NoteBody {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("NoteBody([REDACTED])")
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Kind of CRM note.
    pub enum NoteKind {
        /// Pet handling or care preference.
        PetHandlingPreference,
        /// Communication tone/channel preference.
        CommunicationPreference,
        /// Service preference.
        ServicePreference,
        /// Complaint or service recovery context.
        ServiceRecovery,
        /// Price sensitivity or discount concern.
        PriceSensitivity,
        /// Life event or household context.
        LifeEvent,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Where a candidate CRM signal came from.
    pub enum SignalSource {
        /// Staff observation after an interaction.
        StaffObservation,
        /// Call summary.
        CallSummary,
        /// Checkout/review form.
        CheckoutReviewForm,
        /// Service history inference requiring review.
        ReviewedInference,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Review lifecycle for candidate intelligence.
    pub enum ReviewState {
        /// Candidate fact still awaits review.
        Candidate,
        /// Reviewer accepted the fact.
        Accepted,
        /// Reviewer rejected the fact.
        Rejected,
        /// Fact was superseded by fresher evidence.
        Superseded,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Structured customer/pet note with allowed-use and visibility boundaries.
    pub struct StructuredNote {
        id: NoteId,
        customer_id: entities::CustomerId,
        pet_id: Option<entities::PetId>,
        kind: NoteKind,
        body: NoteBody,
        visibility: access::VisibilityScope,
        #[builder(default)]
        allowed_uses: Vec<access::AllowedUse>,
        source: SignalSource,
        confidence: identity::Confidence,
        review_state: ReviewState,
        recorded_at: DateTime<Utc>,
    }

    impl StructuredNote {
        /// Note id used as segmentation evidence.
        pub const fn id(&self) -> &NoteId {
            &self.id
        }

        /// Returns whether the note may support the requested use.
        pub fn can_support(&self, allowed_use: access::AllowedUse) -> bool {
            self.allowed_uses.contains(&allowed_use)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Customer segment labels supported by reviewed evidence.
    pub enum Segment {
        /// Customer may need service recovery follow-up.
        ServiceRecoveryWatchlist,
        /// Customer appears to value recurring daycare.
        RecurringDaycareCandidate,
        /// Grooming cadence suggests rebooking.
        GroomingRebookCandidate,
        /// Household has boarding peak-period demand.
        HolidayBoardingPlanner,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Evidence basis for a customer segment.
    pub enum SegmentBasis {
        /// Recent complaint or service recovery resolution.
        ComplaintResolvedRecently,
        /// Lapsed visit cadence.
        VisitCadenceDeclined,
        /// Unused package sessions.
        UnusedPackageSessions,
        /// Grooming due date passed.
        GroomingCadenceDue,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Reviewed segment membership with operations/marketing boundary.
    pub struct SegmentMembership {
        customer_id: entities::CustomerId,
        segment: Segment,
        basis: Vec<SegmentBasis>,
        visibility: access::VisibilityScope,
        marketing_allowed: bool,
        #[builder(default)]
        evidence_note_ids: Vec<NoteId>,
    }

    impl SegmentMembership {
        /// Returns whether this segment may be used by marketing.
        pub const fn marketing_allowed(&self) -> bool {
            self.marketing_allowed
        }
    }
}

/// Labor modeling for scheduled coverage and optimization recommendations.
pub mod labor {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Positive labor minute count.
    pub struct Minutes(u16);

    impl Minutes {
        /// Creates a nonzero labor minute count.
        pub const fn try_new(value: u16) -> Result<Self, Error> {
            if value == 0 {
                return Err(Error::ZeroMinutes);
            }
            Ok(Self(value))
        }

        /// Raw minutes.
        pub const fn get(self) -> u16 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Signed minute delta for add/remove/reassign recommendations.
    pub struct SignedMinutes(i32);

    impl SignedMinutes {
        /// Creates a signed minute delta.
        pub const fn new(value: i32) -> Self {
            Self(value)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Positive people count for staffing coverage.
    pub struct PeopleCount(u16);

    impl PeopleCount {
        /// Creates a nonzero people count.
        pub const fn try_new(value: u16) -> Result<Self, Error> {
            if value == 0 {
                return Err(Error::ZeroPeople);
            }
            Ok(Self(value))
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Labor role in a coverage recommendation.
    pub enum Role {
        /// Front desk.
        FrontDesk,
        /// Kennel/care staff.
        KennelTechnician,
        /// Groomer.
        Groomer,
        /// Trainer.
        Trainer,
        /// Manager.
        Manager,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Scheduled labor coverage for a role/time bucket.
    pub struct ScheduledCoverage {
        location_id: entities::LocationId,
        bucket: time::Window,
        role: Role,
        scheduled_people: PeopleCount,
        scheduled_minutes: Minutes,
        loaded_cost_cents: financial::MoneyCents,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Labor validation failure.
    pub enum Error {
        #[error("labor minutes must be greater than zero")]
        /// Zero labor minutes would erase the work requirement.
        ZeroMinutes,
        #[error("people count must be greater than zero")]
        /// Zero people cannot represent coverage.
        ZeroPeople,
    }
}

/// Capacity and demand optimization contracts.
pub mod capacity {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Positive demand quantity.
    pub struct Quantity(u32);

    impl Quantity {
        /// Creates a nonzero quantity.
        pub const fn try_new(value: u32) -> Result<Self, Error> {
            if value == 0 {
                return Err(Error::ZeroQuantity);
            }
            Ok(Self(value))
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Capacity or scheduling constraint.
    pub enum Constraint {
        /// Room or suite inventory constraint.
        RoomOrSuiteAvailability,
        /// Play-yard capacity.
        PlayYardAvailability,
        /// Staff ratio.
        StaffRatio,
        /// Groomer slot capacity.
        GroomerSlotAvailability,
        /// Trainer capacity.
        TrainerAvailability,
        /// Check-in/check-out bottleneck.
        CheckInCheckoutBottleneck,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Time-bucketed service demand and labor standard.
    pub struct DemandUnit {
        location_id: entities::LocationId,
        service: entities::ServiceKind,
        bucket: time::Window,
        quantity: Quantity,
        labor_minutes_per_unit: labor::Minutes,
        #[builder(default)]
        constraints: Vec<Constraint>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Optimization objective.
    pub enum OptimizationObjective {
        /// Reduce front-desk peak queue risk.
        ReduceFrontDeskBottleneck,
        /// Improve utilization without overbooking.
        MaximizeSafeUtilization,
        /// Reduce overstaffing cost.
        ReduceOverstaffing,
        /// Protect service quality during demand peaks.
        ProtectServiceQuality,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Reviewable action recommended by the optimizer.
    pub enum RecommendedAction {
        /// Add role coverage in minutes.
        AddRoleCoverage {
            /// Role to add.
            role: labor::Role,
            /// Minutes to add.
            minutes: labor::Minutes,
        },
        /// Reassign role coverage.
        ReassignCoverage {
            /// Role to reassign.
            role: labor::Role,
            /// Minutes to reassign.
            minutes: labor::Minutes,
        },
        /// Keep plan but flag review.
        ManagerReviewOnly,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Review-gated capacity/labor optimization recommendation.
    pub struct OptimizationRecommendation {
        objective: OptimizationObjective,
        demand: DemandUnit,
        coverage: labor::ScheduledCoverage,
        action: RecommendedAction,
        expected_labor_delta_minutes: labor::SignedMinutes,
        review_gate: policy::ReviewGate,
    }

    impl OptimizationRecommendation {
        /// Required review gate before staffing or scheduling changes.
        pub fn review_gate(&self) -> policy::ReviewGate {
            self.review_gate.clone()
        }

        /// These recommendations never directly change staff schedules.
        pub const fn blocks_live_schedule_change(&self) -> bool {
            true
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Capacity validation failure.
    pub enum Error {
        #[error("demand quantity must be greater than zero")]
        /// Zero demand would erase the optimization target.
        ZeroQuantity,
    }
}

/// Knowledge-base and citation contracts for the Pet Resorts GPT assistant.
pub mod knowledge {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    /// Knowledge document id.
    pub struct DocumentId(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 240),
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
    /// Knowledge document title.
    pub struct Title(String);

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 200),
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
    /// Section reference within a knowledge source.
    pub struct SectionRef(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Kind of knowledge document.
    pub enum DocumentKind {
        /// SOP or operating procedure.
        Sop,
        /// Pricing/service sheet.
        Pricing,
        /// Vendor documentation or contract summary.
        Vendor,
        /// Training document.
        Training,
        /// Local site note approved for assistant use.
        SitePolicy,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Approval/freshness status for knowledge.
    pub enum ApprovalStatus {
        /// Approved for retrieval.
        Approved,
        /// Draft only.
        Draft,
        /// Stale and should be escalated.
        Stale,
        /// Superseded by a newer source.
        Superseded,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Applicability scope for a document.
    pub struct Applicability {
        #[builder(default)]
        locations: Vec<entities::LocationId>,
        #[builder(default)]
        services: Vec<entities::ServiceKind>,
        #[builder(default)]
        roles: Vec<access::ActorRole>,
    }

    impl Applicability {
        fn applies_to(
            &self,
            location_id: entities::LocationId,
            service: entities::ServiceKind,
            role: access::ActorRole,
        ) -> bool {
            self.locations.contains(&location_id)
                && self.services.contains(&service)
                && self.roles.contains(&role)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Approved knowledge document metadata for permissioned retrieval.
    pub struct Document {
        id: DocumentId,
        title: Title,
        kind: DocumentKind,
        status: ApprovalStatus,
        applicability: Applicability,
        effective_at: DateTime<Utc>,
        review_due_at: Option<DateTime<Utc>>,
    }

    impl Document {
        /// Document id used by citations.
        pub const fn id(&self) -> &DocumentId {
            &self.id
        }

        /// Returns whether this document applies to the actor/location/service context.
        pub fn applies_to(
            &self,
            location_id: entities::LocationId,
            service: entities::ServiceKind,
            role: access::ActorRole,
        ) -> bool {
            matches!(self.status, ApprovalStatus::Approved)
                && self.applicability.applies_to(location_id, service, role)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Cited source section used by assistant answers.
    pub struct Citation {
        document_id: DocumentId,
        section: SectionRef,
    }
}

/// Permissioned assistant context and answer packets.
pub mod assistant {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 4000),
        derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
    )]
    /// Assistant answer text.
    pub struct AnswerText(String);

    impl fmt::Debug for AnswerText {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("AnswerText([REDACTED])")
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Purpose of the assistant interaction.
    pub enum Purpose {
        /// SOP lookup.
        SopLookup,
        /// Pricing or service explanation.
        PricingQuestion,
        /// Vendor information lookup.
        VendorLookup,
        /// Site operational decision support.
        SiteOpsSupport,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Reason an assistant response should escalate.
    pub enum EscalationReason {
        /// Source conflict.
        ConflictingSources,
        /// Stale/missing source.
        StaleOrMissingSource,
        /// Sensitive customer, pet, financial, or safety context.
        SensitiveContext,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Actor/location/purpose scope for permissioned assistant retrieval.
    pub struct ActorContext {
        actor_id: access::ActorId,
        role: access::ActorRole,
        title: access::Title,
        location_id: entities::LocationId,
        purpose: Purpose,
        #[builder(default)]
        allowed_uses: Vec<access::AllowedUse>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Assistant answer with citations and escalation state.
    pub struct AnswerPacket {
        context: ActorContext,
        answer: AnswerText,
        #[builder(default)]
        citations: Vec<knowledge::Citation>,
        confidence: identity::Confidence,
        escalation: Option<EscalationReason>,
    }

    impl AnswerPacket {
        /// Returns whether answer contains at least one citation and no missing-source escalation.
        pub fn is_cited(&self) -> bool {
            !self.citations.is_empty()
                && !matches!(
                    self.escalation,
                    Some(EscalationReason::StaleOrMissingSource)
                )
        }
    }
}

/// Site financial facts, KPIs, and review-gated recommendations.
pub mod financial {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    /// Money amount in cents.
    pub struct MoneyCents(i64);

    impl MoneyCents {
        /// Creates a non-negative cents value, panicking if a caller attempts to encode a negative financial fact.
        pub const fn new(value: i64) -> Self {
            if value < 0 {
                panic!("money cents cannot be negative");
            }
            Self(value)
        }

        /// Tries to create a non-negative cents value without panicking.
        pub const fn try_new(value: i64) -> Result<Self, Error> {
            if value < 0 {
                return Err(Error::NegativeMoney);
            }
            Ok(Self(value))
        }

        /// Raw cents value.
        pub const fn get(self) -> i64 {
            self.0
        }
    }

    impl<'de> Deserialize<'de> for MoneyCents {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = i64::deserialize(deserializer)?;
            Self::try_new(value).map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Reporting period for one site.
    pub struct SitePeriod {
        location_id: entities::LocationId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    }

    impl SitePeriod {
        /// Starts a manual builder that validates start/end order.
        pub const fn builder() -> SitePeriodBuilder {
            SitePeriodBuilder::new()
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    /// Builder for a site financial reporting period.
    pub struct SitePeriodBuilder {
        location_id: Option<entities::LocationId>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    }

    impl SitePeriodBuilder {
        /// Creates an empty builder.
        pub const fn new() -> Self {
            Self {
                location_id: None,
                start: None,
                end: None,
            }
        }

        /// Sets location id.
        pub const fn location_id(mut self, value: entities::LocationId) -> Self {
            self.location_id = Some(value);
            self
        }

        /// Sets period start.
        pub const fn start(mut self, value: DateTime<Utc>) -> Self {
            self.start = Some(value);
            self
        }

        /// Sets period end.
        pub const fn end(mut self, value: DateTime<Utc>) -> Self {
            self.end = Some(value);
            self
        }

        /// Builds a period if all required values exist and end follows start.
        pub fn build(self) -> Result<SitePeriod, Error> {
            let location_id = self.location_id.ok_or(Error::MissingLocation)?;
            let start = self.start.ok_or(Error::MissingStart)?;
            let end = self.end.ok_or(Error::MissingEnd)?;
            if end <= start {
                return Err(Error::EndMustFollowStart);
            }
            Ok(SitePeriod {
                location_id,
                start,
                end,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Source-backed site/service financial fact.
    pub struct RevenueFact {
        period: SitePeriod,
        service: entities::ServiceKind,
        gross_revenue_cents: MoneyCents,
        discount_cents: MoneyCents,
        refund_cents: MoneyCents,
        labor_cost_cents: MoneyCents,
        source_system: source::System,
    }

    impl RevenueFact {
        /// Net revenue after discounts and refunds.
        pub const fn net_revenue_cents(&self) -> MoneyCents {
            MoneyCents::new(
                self.gross_revenue_cents.get()
                    - self.discount_cents.get()
                    - self.refund_cents.get(),
            )
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Site financial insight kind.
    pub enum InsightKind {
        /// Discount leakage appears elevated.
        DiscountLeakage,
        /// Refund rate appears elevated.
        RefundRateVariance,
        /// Labor percent appears elevated.
        LaborCostVariance,
        /// Add-on attach rate opportunity.
        AddOnAttachRateOpportunity,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Review-gated financial recommendation.
    pub enum Recommendation {
        /// Review discount policy or approvals.
        ReviewDiscountPolicy,
        /// Review labor plan.
        ReviewLaborPlan,
        /// Review add-on attach process.
        ReviewAddOnWorkflow,
        /// Investigate source/accounting variance.
        InvestigateVariance,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Review-gated site financial insight.
    pub struct Insight {
        fact: RevenueFact,
        kind: InsightKind,
        expected_impact_cents: MoneyCents,
        recommendation: Recommendation,
        review_gate: policy::ReviewGate,
    }

    impl Insight {
        /// Net revenue on the underlying fact.
        pub const fn net_revenue_cents(&self) -> MoneyCents {
            self.fact.net_revenue_cents()
        }

        /// Financial insight cannot directly mutate price, discount, payment, or accounting state.
        pub const fn blocks_financial_mutation(&self) -> bool {
            true
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Site-period validation failure.
    pub enum Error {
        #[error("money cents cannot be negative")]
        /// Financial amounts in strategic insight facts must be non-negative.
        NegativeMoney,
        #[error("financial site period requires a location")]
        /// Location id missing.
        MissingLocation,
        #[error("financial site period requires a start")]
        /// Start missing.
        MissingStart,
        #[error("financial site period requires an end")]
        /// End missing.
        MissingEnd,
        #[error("financial site period end must follow start")]
        /// End did not follow start.
        EndMustFollowStart,
    }
}

/// Generalized outcome attribution for strategic AI-ops value claims.
pub mod outcome {
    use super::*;

    #[nutype(
        sanitize(trim),
        validate(not_empty, len_char_max = 160),
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
    /// Outcome record id.
    pub struct Id(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Strategic AI-ops workstream.
    pub enum Workstream {
        /// Real-time lead response.
        LeadResponse,
        /// Capacity/labor optimization.
        CapacityLabor,
        /// Knowledge assistant.
        KnowledgeAssistant,
        /// Retention.
        Retention,
        /// Site financial insights.
        FinancialInsights,
        /// CRM note intelligence.
        CrmIntelligence,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Value metric tracked by an outcome.
    pub enum Metric {
        /// Booking converted.
        BookingConverted,
        /// Labor minutes saved.
        LaborMinutesSaved,
        /// Utilization basis points improved.
        UtilizationBasisPoints,
        /// Revenue cents improved.
        RevenueCents,
        /// Customer retained.
        CustomerRetained,
        /// Handle time reduced.
        HandleTimeMinutesReduced,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Metric value representation.
    pub enum MetricValue {
        /// Count value.
        Count(i64),
        /// Minutes value.
        Minutes(i64),
        /// Cents value.
        Cents(i64),
        /// Basis points value.
        BasisPoints(i64),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Attribution strength for a value claim.
    pub enum Attribution {
        /// Human-reviewed action can support value claims.
        ReviewedAction,
        /// Correlated with recommendation but not enough for strong claims.
        CorrelatedOnly,
        /// Source was wrong, so no value claim is allowed.
        WrongSource,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Source-backed outcome record for business-value claims.
    pub struct Record {
        id: Id,
        workstream: Workstream,
        location_id: entities::LocationId,
        metric: Metric,
        before_value: MetricValue,
        after_value: MetricValue,
        attribution: Attribution,
        source: source::System,
        recorded_at: DateTime<Utc>,
    }

    impl Record {
        /// Workstream this outcome belongs to.
        pub const fn workstream(&self) -> Workstream {
            self.workstream
        }

        /// Returns whether this outcome can support a value claim.
        pub const fn can_support_value_claim(&self) -> bool {
            matches!(self.attribution, Attribution::ReviewedAction)
        }
    }
}
