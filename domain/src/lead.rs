//! Cross-service lead conversion triage for resort sales follow-up.
//!
//! These types describe resort sales/intake state independent of any one
//! service line. They promote web/phone/SMS/source facts into validated sales
//! workflow state so follow-up labor, booking readiness, and revenue opportunities
//! are visible without letting an agent invent availability or bypass staff review.

use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::entities::{CustomerId, ServiceKind};
use crate::{
    consent as communication, entities, identity, message, money, operations, policy, source,
};

/// Review-gated realtime lead-response packets and SLA evidence.
///
/// Canonical owner for lead response concepts previously introduced by
/// `strategic_ai_ops::lead_response`. These values remain source-backed and do not authorize live
/// customer sends, provider writes, booking promises, or schedule mutations.
///
/// The promotion chain is deliberately narrow: a source event plus SLA evidence, exact
/// channel/purpose consent, and an ordered reviewed attempt can become a packet; only a matching
/// customer-message approval can produce a queueable action, and that proof token still exposes no
/// live-send method.
///
/// ```
/// use chrono::{TimeZone, Utc};
/// use domain::{consent, entities, identity, lead, message, money, policy, source};
///
/// let received_at = Utc.with_ymd_and_hms(2026, 8, 12, 14, 0, 0).unwrap();
/// let message_ref = message::BodyRef::try_new("draft-body-42")?;
/// let event = lead::response::Event::builder()
///     .id(lead::response::EventId::try_new("missed-call-42")?)
///     .idempotency_key(lead::response::IdempotencyKey::try_new("masked-source-key")?)
///     .location_id(entities::LocationId(uuid::Uuid::nil()))
///     .kind(lead::response::EventKind::MissedCall)
///     .received_at(received_at)
///     .source_system(source::System::Telephony)
///     .customer_match(identity::Match::Candidate {
///         customer_id: entities::CustomerId(uuid::Uuid::from_u128(42)),
///         confidence: identity::Confidence::High,
///     })
///     .service_intent(entities::ServiceKind::Boarding)
///     .build();
/// let sla = lead::response::ResponseSla::try_new(
///     lead::response::SlaTarget::FirstResponseWithinMinutes(
///         lead::response::Minutes::try_new(5)?,
///     ),
///     received_at,
///     received_at + chrono::Duration::minutes(5),
///     lead::response::SlaStatus::Open,
/// )?;
/// let attempt = lead::response::ContactAttempt::try_new(
///     received_at + chrono::Duration::minutes(2),
///     consent::Channel::Sms,
///     consent::Purpose::TransactionalLeadResponse,
///     lead::response::AttemptOutcome::DraftedForReview,
///     policy::ReviewGate::CustomerMessageApproval,
/// )?
/// .with_message_ref(message_ref.clone());
/// let packet = lead::response::ResponsePacket::try_new(
///     event,
///     sla,
///     consent::ConsentEvidence::builder()
///         .channel(consent::Channel::Sms)
///         .purpose(consent::Purpose::TransactionalLeadResponse)
///         .status(consent::ConsentStatus::Granted)
///         .source(source::System::Crm)
///         .build(),
///     vec![attempt],
///     lead::response::ConversionAttribution::builder()
///         .source(lead::response::AttributionSource::MissedCall)
///         .estimated_value(money::Money::usd(45_000)?)
///         .build(),
/// )?;
/// let approval = lead::response::ReviewApproval::builder()
///     .gate(policy::ReviewGate::CustomerMessageApproval)
///     .location_id(entities::LocationId(uuid::Uuid::nil()))
///     .customer_id(entities::CustomerId(uuid::Uuid::from_u128(42)))
///     .service_intent(entities::ServiceKind::Boarding)
///     .channel(consent::Channel::Sms)
///     .purpose(consent::Purpose::TransactionalLeadResponse)
///     .message_ref(message_ref)
///     .build();
/// let queueable = lead::response::QueueableContact::try_from_review(&packet, approval)?;
/// assert_eq!(queueable.action(), lead::response::LegalContactAction::QueueOnly);
/// assert!(queueable.live_send_is_unavailable());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub mod response {
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
        derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
    )]
    /// Idempotency key used to deduplicate replayed lead events.
    pub struct IdempotencyKey(String);

    impl fmt::Debug for IdempotencyKey {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("IdempotencyKey([REDACTED])")
        }
    }

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

    impl<'de> Deserialize<'de> for Minutes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = u16::deserialize(deserializer)?;
            Self::try_new(value).map_err(serde::de::Error::custom)
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
        /// Idempotency key retained for replay/deduplication without exposing it in debug output.
        pub const fn idempotency_key(&self) -> &IdempotencyKey {
            &self.idempotency_key
        }

        /// Location where the lead response event occurred.
        pub const fn location_id(&self) -> entities::LocationId {
            self.location_id
        }

        /// Event receipt timestamp that anchors SLA and attempt ordering.
        pub const fn received_at(&self) -> DateTime<Utc> {
            self.received_at
        }

        /// Identity match supplied by source/customer reconciliation.
        pub const fn customer_match(&self) -> &identity::Match {
            &self.customer_match
        }

        /// Service intent inferred from source evidence.
        pub fn service_intent(&self) -> entities::ServiceKind {
            self.service_intent.clone()
        }

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

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Lead-response SLA evidence.
    pub struct ResponseSla {
        target: SlaTarget,
        received_at: DateTime<Utc>,
        due_at: DateTime<Utc>,
        status: SlaStatus,
    }

    impl ResponseSla {
        /// Creates SLA evidence only when the due timestamp agrees with the receipt timestamp and target.
        pub fn try_new(
            target: SlaTarget,
            received_at: DateTime<Utc>,
            due_at: DateTime<Utc>,
            status: SlaStatus,
        ) -> Result<Self, Error> {
            let expected_due_at = match target {
                SlaTarget::FirstResponseWithinMinutes(minutes) => {
                    received_at + chrono::Duration::minutes(i64::from(minutes.get()))
                }
            };
            if due_at != expected_due_at {
                return Err(Error::SlaDueAtDoesNotMatchTarget);
            }
            Ok(Self {
                target,
                received_at,
                due_at,
                status,
            })
        }

        /// Starts building SLA evidence with relationship validation at build time.
        pub const fn builder() -> ResponseSlaBuilder {
            ResponseSlaBuilder::new()
        }

        /// Receipt timestamp used to validate attempts against the event.
        pub const fn received_at(&self) -> DateTime<Utc> {
            self.received_at
        }

        /// Returns whether an attempted first response occurred before the due time.
        pub fn is_met_at(&self, at: DateTime<Utc>) -> bool {
            matches!(self.status, SlaStatus::Open | SlaStatus::Met)
                && at >= self.received_at
                && at <= self.due_at
        }
    }

    /// Builder for SLA evidence that validates target/due timestamp agreement.
    pub struct ResponseSlaBuilder {
        target: Option<SlaTarget>,
        received_at: Option<DateTime<Utc>>,
        due_at: Option<DateTime<Utc>>,
        status: Option<SlaStatus>,
    }

    impl Default for ResponseSlaBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ResponseSlaBuilder {
        /// Creates an empty SLA builder.
        pub const fn new() -> Self {
            Self {
                target: None,
                received_at: None,
                due_at: None,
                status: None,
            }
        }

        /// Sets the SLA target.
        pub fn target(mut self, target: SlaTarget) -> Self {
            self.target = Some(target);
            self
        }
        /// Sets when the lead event was received.
        pub fn received_at(mut self, received_at: DateTime<Utc>) -> Self {
            self.received_at = Some(received_at);
            self
        }
        /// Sets the computed SLA due timestamp.
        pub fn due_at(mut self, due_at: DateTime<Utc>) -> Self {
            self.due_at = Some(due_at);
            self
        }
        /// Sets the current SLA status.
        pub fn status(mut self, status: SlaStatus) -> Self {
            self.status = Some(status);
            self
        }

        /// Builds the SLA if all required fields exist and the due timestamp matches the target.
        pub fn build(self) -> Result<ResponseSla, Error> {
            ResponseSla::try_new(
                self.target.ok_or(Error::MissingSlaField)?,
                self.received_at.ok_or(Error::MissingSlaField)?,
                self.due_at.ok_or(Error::MissingSlaField)?,
                self.status.ok_or(Error::MissingSlaField)?,
            )
        }
    }

    impl<'de> Deserialize<'de> for ResponseSla {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawResponseSla {
                target: SlaTarget,
                received_at: DateTime<Utc>,
                due_at: DateTime<Utc>,
                status: SlaStatus,
            }
            let raw = RawResponseSla::deserialize(deserializer)?;
            Self::try_new(raw.target, raw.received_at, raw.due_at, raw.status)
                .map_err(serde::de::Error::custom)
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
        message_ref: Option<message::BodyRef>,
    }

    impl ContactAttempt {
        /// Creates a contact attempt with no linked draft message yet.
        pub fn try_new(
            attempted_at: DateTime<Utc>,
            channel: communication::Channel,
            purpose: communication::Purpose,
            outcome: AttemptOutcome,
            review_gate: policy::ReviewGate,
        ) -> Result<Self, Error> {
            Ok(Self {
                attempted_at,
                channel,
                purpose,
                outcome,
                review_gate,
                message_ref: None,
            })
        }

        /// Returns this attempt linked to its reviewed message draft reference.
        pub fn with_message_ref(mut self, message_ref: message::BodyRef) -> Self {
            self.message_ref = Some(message_ref);
            self
        }

        /// Attempt timestamp used for ordering and SLA checks.
        pub const fn attempted_at(&self) -> DateTime<Utc> {
            self.attempted_at
        }
        /// Attempt channel that must be exactly covered by consent and approval evidence.
        pub const fn channel(&self) -> communication::Channel {
            self.channel
        }
        /// Attempt purpose that must be exactly covered by consent and approval evidence.
        pub const fn purpose(&self) -> communication::Purpose {
            self.purpose
        }
        /// Linked message draft body reference, if an outbound draft exists.
        pub const fn message_ref(&self) -> Option<&message::BodyRef> {
            self.message_ref.as_ref()
        }

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
        estimated_value: Option<money::Money>,
    }

    impl ConversionAttribution {
        /// Creates attribution for a converted lead; conversion claims require reservation evidence.
        pub fn try_converted(
            source: AttributionSource,
            campaign: Option<Campaign>,
            converted_reservation_id: Option<entities::reservation::Id>,
            estimated_value: Option<money::Money>,
        ) -> Result<Self, Error> {
            if converted_reservation_id.is_none() {
                return Err(Error::ConvertedLeadRequiresReservation);
            }
            Ok(Self {
                source,
                campaign,
                converted_reservation_id,
                estimated_value,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Reviewable lead response packet.
    pub struct ResponsePacket {
        event: Event,
        sla: ResponseSla,
        consent: communication::ConsentEvidence,
        attempts: Vec<ContactAttempt>,
        attribution: ConversionAttribution,
    }

    impl ResponsePacket {
        /// Creates a packet only after event, SLA, consent, attempt, and attribution relationships agree.
        pub fn try_new(
            event: Event,
            sla: ResponseSla,
            consent: communication::ConsentEvidence,
            attempts: Vec<ContactAttempt>,
            attribution: ConversionAttribution,
        ) -> Result<Self, Error> {
            if sla.received_at() != event.received_at() {
                return Err(Error::SlaReceiptDoesNotMatchEvent);
            }
            if attempts.is_empty() {
                return Err(Error::MissingContactAttempt);
            }
            let mut previous_attempt_at: Option<DateTime<Utc>> = None;
            for attempt in &attempts {
                if attempt.attempted_at() < event.received_at() {
                    return Err(Error::ContactAttemptBeforeReceipt);
                }
                if previous_attempt_at.is_some_and(|previous| attempt.attempted_at() < previous) {
                    return Err(Error::ContactAttemptsOutOfOrder);
                }
                if !consent.permits(attempt.channel(), attempt.purpose()) {
                    return Err(Error::ConsentDoesNotCoverAttempt);
                }
                previous_attempt_at = Some(attempt.attempted_at());
            }
            Ok(Self {
                event,
                sla,
                consent,
                attempts,
                attribution,
            })
        }

        /// Starts a packet builder that validates cross-field invariants at build time.
        pub const fn builder() -> ResponsePacketBuilder {
            ResponsePacketBuilder::new()
        }

        /// Lead event in the packet.
        pub const fn event(&self) -> &Event {
            &self.event
        }
        /// Attempts retained on the packet.
        pub fn attempts(&self) -> &[ContactAttempt] {
            &self.attempts
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

    /// Builder for lead-response packets that validates cross-field relationships at build time.
    pub struct ResponsePacketBuilder {
        event: Option<Event>,
        sla: Option<ResponseSla>,
        consent: Option<communication::ConsentEvidence>,
        attempts: Option<Vec<ContactAttempt>>,
        attribution: Option<ConversionAttribution>,
    }

    impl Default for ResponsePacketBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ResponsePacketBuilder {
        /// Creates an empty packet builder.
        pub const fn new() -> Self {
            Self {
                event: None,
                sla: None,
                consent: None,
                attempts: None,
                attribution: None,
            }
        }
        /// Sets the lead source event.
        pub fn event(mut self, event: Event) -> Self {
            self.event = Some(event);
            self
        }
        /// Sets the response SLA.
        pub fn sla(mut self, sla: ResponseSla) -> Self {
            self.sla = Some(sla);
            self
        }
        /// Sets the consent evidence used to validate attempts.
        pub fn consent(mut self, consent: communication::ConsentEvidence) -> Self {
            self.consent = Some(consent);
            self
        }
        /// Sets the reviewed contact attempts.
        pub fn attempts(mut self, attempts: Vec<ContactAttempt>) -> Self {
            self.attempts = Some(attempts);
            self
        }
        /// Sets conversion attribution evidence.
        pub fn attribution(mut self, attribution: ConversionAttribution) -> Self {
            self.attribution = Some(attribution);
            self
        }

        /// Builds the packet after validating cross-field relationships.
        pub fn build(self) -> Result<ResponsePacket, Error> {
            ResponsePacket::try_new(
                self.event.ok_or(Error::MissingPacketField)?,
                self.sla.ok_or(Error::MissingPacketField)?,
                self.consent.ok_or(Error::MissingPacketField)?,
                self.attempts.unwrap_or_default(),
                self.attribution.ok_or(Error::MissingPacketField)?,
            )
        }
    }

    impl<'de> Deserialize<'de> for ResponsePacket {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawResponsePacket {
                event: Event,
                sla: ResponseSla,
                consent: communication::ConsentEvidence,
                attempts: Vec<ContactAttempt>,
                attribution: ConversionAttribution,
            }
            let raw = RawResponsePacket::deserialize(deserializer)?;
            Self::try_new(
                raw.event,
                raw.sla,
                raw.consent,
                raw.attempts,
                raw.attribution,
            )
            .map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Legal lead-contact action permitted by the domain lifecycle.
    pub enum LegalContactAction {
        /// Approved material may be queued; live send remains unavailable.
        QueueOnly,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Review proof that can promote a packet into a queueable, not live-sendable, contact action.
    pub struct ReviewApproval {
        gate: policy::ReviewGate,
        location_id: entities::LocationId,
        customer_id: entities::CustomerId,
        service_intent: entities::ServiceKind,
        channel: communication::Channel,
        purpose: communication::Purpose,
        message_ref: message::BodyRef,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Proof token for an approved outbound draft that can only enter a queue.
    pub struct QueueableContact {
        action: LegalContactAction,
        approval: ReviewApproval,
    }

    impl QueueableContact {
        /// Promotes a packet only when review approval matches location, customer, service, channel, purpose, and message draft.
        pub fn try_from_review(
            packet: &ResponsePacket,
            approval: ReviewApproval,
        ) -> Result<Self, Error> {
            let event_customer_id = match packet.event().customer_match() {
                identity::Match::Candidate { customer_id, .. } => *customer_id,
                identity::Match::None | identity::Match::Ambiguous { .. } => {
                    return Err(Error::ReviewApprovalDoesNotMatchPacket);
                }
            };
            let matched_attempt = packet.attempts().iter().any(|attempt| {
                attempt.channel() == approval.channel
                    && attempt.purpose() == approval.purpose
                    && attempt.review_gate() == approval.gate
                    && attempt.message_ref() == Some(&approval.message_ref)
            });
            if approval.gate != policy::ReviewGate::CustomerMessageApproval
                || approval.location_id != packet.event().location_id()
                || approval.customer_id != event_customer_id
                || approval.service_intent != packet.event().service_intent()
                || !matched_attempt
            {
                return Err(Error::ReviewApprovalDoesNotMatchPacket);
            }
            Ok(Self {
                action: LegalContactAction::QueueOnly,
                approval,
            })
        }

        /// Action exposed by the proof token.
        pub const fn action(&self) -> LegalContactAction {
            self.action
        }
        /// Live customer send is intentionally unavailable in this pilot domain surface.
        pub const fn live_send_is_unavailable(&self) -> bool {
            true
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Lead-response validation error.
    pub enum Error {
        #[error("minutes must be greater than zero")]
        /// Zero minutes would erase the SLA meaning.
        ZeroMinutes,
        #[error("SLA due timestamp must match event receipt plus target")]
        /// SLA due timestamp did not agree with its receipt timestamp and target minutes.
        SlaDueAtDoesNotMatchTarget,
        #[error("SLA receipt timestamp must match the lead event receipt timestamp")]
        /// Packet-level SLA receipt and event receipt did not agree.
        SlaReceiptDoesNotMatchEvent,
        #[error("contact attempt cannot occur before lead event receipt")]
        /// A contact attempt predates the source event.
        ContactAttemptBeforeReceipt,
        #[error("contact attempts must be ordered by attempted_at")]
        /// Contact attempts were supplied out of chronological order.
        ContactAttemptsOutOfOrder,
        #[error("consent must cover the exact attempt channel and purpose")]
        /// Consent was missing, opted out, or mismatched for attempt channel/purpose.
        ConsentDoesNotCoverAttempt,
        #[error(
            "review approval does not match packet location, customer, service, channel, purpose, gate, or message"
        )]
        /// Review proof did not bind to the same packet facts and message draft.
        ReviewApprovalDoesNotMatchPacket,
        #[error("converted lead attribution requires a converted reservation id")]
        /// Conversion attribution cannot claim conversion without reservation evidence.
        ConvertedLeadRequiresReservation,
        #[error("missing required SLA field")]
        /// SLA builder lacked a required field.
        MissingSlaField,
        #[error("missing required response packet field")]
        /// Packet builder lacked a required field.
        MissingPacketField,
        #[error("lead-response packet requires at least one reviewed contact attempt")]
        /// A packet without an attempt cannot prove response/review behavior.
        MissingContactAttempt,
    }
}

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
/// Validated local-referral/source name for lead provenance.
pub struct SourceName(String);

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
/// Validated campaign name used to connect lead work to marketing sources.
pub struct CampaignName(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Lead triage record that turns source contact evidence into safe sales follow-up work.
pub struct Triage {
    /// Existing customer record when staff can link the lead to a known account.
    pub customer_id: Option<CustomerId>,
    /// Channel or campaign that explains where the lead came from.
    pub source: Source,
    /// Service or change the customer appears to be asking about.
    pub intent: Intent,
    /// Sales stage used to rank follow-up labor and booking readiness.
    pub stage: ConversionStage,
    /// Requested resort service when the source evidence is specific enough.
    pub requested_service: Option<ServiceKind>,
    /// Staff-safe next step; automation may draft, route, or summarize but not book or promise capacity.
    pub next_action: NextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Lead source retained so marketing and intake teams can audit where demand originated.
pub enum Source {
    /// Lead originated from a website form and may be routed to intake follow-up.
    WebsiteForm,
    /// Lead originated from a phone call or voicemail that staff may need to summarize or return.
    Phone,
    /// Lead originated from SMS and should respect texting consent and response boundaries.
    Sms,
    /// Lead originated from email and can support draft replies after staff-safe triage.
    Email,
    /// Lead originated from social media and may need attribution or identity verification.
    SocialMedia,
    /// Local referral name staff can verify before attributing the lead source.
    LocalReferral {
        /// Referral source name preserved for staff attribution and deduplication.
        source_name: SourceName,
    },
    /// Contact or display name used by staff.
    Campaign {
        /// Campaign name preserved for marketing attribution and follow-up reporting.
        name: CampaignName,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Prospect intent signal used to route boarding, daycare, grooming, or training follow-up.
pub enum Intent {
    /// New prospect asking about onboarding, requirements, availability, or first booking.
    NewCustomerIntake,
    /// Prospect wants boarding pricing or availability that staff must confirm before promising.
    BoardingQuote,
    /// Prospect is asking about daycare trial or evaluation; staff must confirm eligibility steps.
    DaycareTrial,
    /// Prospect wants grooming scheduling, which depends on service, pet, and capacity review.
    GroomingAppointment,
    /// Prospect wants training consultation routed to the appropriate trainer or intake path.
    TrainingConsult,
    /// Existing customer appears to need a booking or profile change rather than new intake.
    ExistingCustomerChange,
    /// Lead intent is unclear; automation may summarize but staff must classify before booking promises.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Conversion stage that separates new inquiries from booked, lost, or inactive demand.
pub enum ConversionStage {
    /// New lead awaiting first staff or automated draft response.
    New,
    /// Staff or automation attempted contact and the next step depends on response evidence.
    ContactAttempted,
    /// Lead is paused until the customer supplies missing answers or documents.
    WaitingOnCustomer,
    /// Lead cannot be booked until vaccine, pet profile, policy, or other requirements are confirmed.
    MissingRequirements,
    /// Intake evidence looks booking-ready, but staff must still confirm capacity and policies before committing.
    ReadyToBook,
    /// Lead has converted into booked or active customer work and should avoid duplicate sales follow-up.
    Converted,
    /// Lead is inactive or declined, retained for attribution and future analysis.
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Human-safe next step for converting a lead without overpromising capacity or policy.
pub enum NextAction {
    /// Automation may draft a reply, but sending still follows channel and approval gates.
    DraftReply,
    /// Ask for pet profile details needed before eligibility or booking review.
    RequestMissingPetProfile,
    /// Ask for vaccine proof before trial, daycare, boarding, or grooming readiness decisions.
    RequestVaccineProof,
    /// Staff-confirmed availability can be offered; automation must not invent or hold times.
    OfferReservationTimes,
    /// Route to staff when source facts, policy, or customer context are too ambiguous for automation.
    RouteToHuman {
        /// Business reason staff should review before proceeding.
        reason: operations::operational::Observation,
    },
    /// No current follow-up is appropriate, usually because the lead is converted, lost, or waiting.
    NoAction,
}
