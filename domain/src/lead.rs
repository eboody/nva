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

use crate::{consent as communication, entities, identity, message, money, policy, source};

/// Review-gated realtime lead-response packets and SLA evidence.
///
/// Canonical owner for lead response concepts previously introduced by
/// `strategic_ai_ops::lead_response`. These values remain source-backed and do not authorize live
/// customer sends, provider writes, booking promises, or schedule mutations.
///
/// The current public chain is deliberately fail-closed: source events, caller-reported SLA labels,
/// consent evidence, and attempt labels remain serializable history but cannot construct a response
/// packet. No contact, review, SLA, queue, conversion, or value-attribution authority exists until a
/// future authenticated acceptance boundary issues opaque, exact authority.
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
///     .location_id(entities::LocationId::new(uuid::Uuid::from_u128(1)))
///     .kind(lead::response::EventKind::MissedCall)
///     .received_at(received_at)
///     .source_system(source::System::Telephony)
///     .customer_match(identity::Match::Candidate {
///         customer_id: entities::CustomerId::new(uuid::Uuid::from_u128(42)),
///         confidence: identity::Confidence::High,
///     })
///     .service_intent(entities::ServiceKind::Boarding)
///     .build();
/// let consent_subject = event.source_record();
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
///         .subject(consent::Subject::SourceRecord(consent_subject))
///         .source_record(source::RecordRef::new(
///             source::System::Crm,
///             source::record::Id::try_new("consent-missed-call-42")?,
///         ))
///         .source_schema_version(source::SchemaVersion::try_new("crm-consent-v1")?)
///         .effective_from(received_at - chrono::Duration::days(1))
///         .effective_until(received_at + chrono::Duration::days(1))
///         .build(),
///     vec![attempt],
///     lead::response::ConversionObservation::builder()
///         .source(lead::response::AttributionSource::MissedCall)
///         .estimated_value(money::Money::usd(45_000)?)
///         .build(),
/// );
/// assert!(matches!(
///     packet,
///     Err(lead::response::Error::ConsentDoesNotCoverAttempt)
/// ));
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

        /// Stable source record that owns this lead event and must match consent evidence.
        pub fn source_record(&self) -> source::RecordRef {
            source::RecordRef::new(
                self.source_system,
                source::record::Id::try_new(self.id.clone().into_inner())
                    .expect("validated lead event ids satisfy source record id bounds"),
            )
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// SLA target for lead response.
    pub enum SlaTarget {
        /// First response should happen within this number of minutes.
        FirstResponseWithinMinutes(Minutes),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Caller-reported SLA observation; no variant proves response, contact, review, or SLA attainment.
    pub enum SlaStatus {
        /// Caller reports the SLA as open.
        Open,
        /// Caller reports an SLA-met label; this does not prove a response or SLA attainment.
        Met,
        /// Caller reports an SLA-breached label; this does not prove chronology or adjudication.
        Breached,
        /// Caller reports an after-hours-paused label; this does not prove policy application.
        PausedAfterHours,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Caller-reported lead-response SLA evidence; it cannot establish response or SLA attainment.
    pub struct ResponseSla {
        target: SlaTarget,
        received_at: DateTime<Utc>,
        due_at: DateTime<Utc>,
        status: SlaStatus,
    }

    impl ResponseSla {
        /// Retains caller-reported SLA evidence only when its due timestamp is arithmetically coherent with its receipt timestamp and target; coherence proves no response or SLA attainment.
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
    /// Caller-reported booking observation retained as evidence, never accepted value attribution.
    pub struct ConversionObservation {
        source: AttributionSource,
        campaign: Option<Campaign>,
        converted_reservation_id: Option<entities::reservation::Id>,
        estimated_value: Option<money::Money>,
    }

    impl ConversionObservation {
        /// Creates a reported booking observation; this remains non-claimable serializable evidence.
        pub fn try_reported_booking_observation(
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

        /// Serializable booking observations never support conversion or revenue attribution claims.
        pub const fn can_support_value_claim(&self) -> bool {
            false
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Reviewable lead response packet.
    pub struct ResponsePacket {
        event: Event,
        sla: ResponseSla,
        consent: communication::ConsentEvidence,
        attempts: Vec<ContactAttempt>,
        attribution: ConversionObservation,
    }

    impl ResponsePacket {
        /// Attempts to create a packet after structural validation.
        ///
        /// Caller-constructible consent evidence cannot authorize contact, so production issuance
        /// remains unavailable until a source-authenticated accepted-consent adapter exists.
        pub fn try_new(
            event: Event,
            sla: ResponseSla,
            consent: communication::ConsentEvidence,
            attempts: Vec<ContactAttempt>,
            attribution: ConversionObservation,
        ) -> Result<Self, Error> {
            Self::try_new_checked(event, sla, consent, attempts, attribution, |_, _, _, _| {
                false
            })
        }

        #[cfg(test)]
        fn try_new_with_accepted_consent(
            event: Event,
            sla: ResponseSla,
            accepted: communication::AcceptedConsent,
            attempts: Vec<ContactAttempt>,
            attribution: ConversionObservation,
        ) -> Result<Self, Error> {
            let consent = accepted.evidence().clone();
            Self::try_new_checked(
                event,
                sla,
                consent,
                attempts,
                attribution,
                |subject, channel, purpose, as_of| {
                    accepted.permits_source_record(subject, channel, purpose, as_of)
                },
            )
        }

        fn try_new_checked(
            event: Event,
            sla: ResponseSla,
            consent: communication::ConsentEvidence,
            attempts: Vec<ContactAttempt>,
            attribution: ConversionObservation,
            permits: impl Fn(
                &source::RecordRef,
                communication::Channel,
                communication::Purpose,
                DateTime<Utc>,
            ) -> bool,
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
                previous_attempt_at = Some(attempt.attempted_at());
            }
            for attempt in &attempts {
                if !permits(
                    &event.source_record(),
                    attempt.channel(),
                    attempt.purpose(),
                    attempt.attempted_at(),
                ) {
                    return Err(Error::ConsentDoesNotCoverAttempt);
                }
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

        /// Returns whether any attempt remains behind a customer-message approval gate.
        pub fn requires_customer_message_approval(&self) -> bool {
            self.attempts
                .iter()
                .any(|attempt| attempt.review_gate() == policy::ReviewGate::CustomerMessageApproval)
        }

        /// Returns false because retained caller evidence cannot grant an outbound response.
        pub fn consent_allows_response(
            &self,
            channel: communication::Channel,
            purpose: communication::Purpose,
        ) -> bool {
            self.consent.permits_source_record(
                &self.event.source_record(),
                channel,
                purpose,
                self.event.received_at(),
            )
        }
    }

    /// Builder for lead-response packets that validates cross-field relationships at build time.
    pub struct ResponsePacketBuilder {
        event: Option<Event>,
        sla: Option<ResponseSla>,
        consent: Option<communication::ConsentEvidence>,
        attempts: Option<Vec<ContactAttempt>>,
        attribution: Option<ConversionObservation>,
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
        /// Sets caller-reported consent evidence retained for fail-closed validation.
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
        pub fn attribution(mut self, attribution: ConversionObservation) -> Self {
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
                attribution: ConversionObservation,
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

    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
    /// Serializable caller-reported review labels; they authenticate no actor, review, contact, message, or queue authority.
    pub struct ReviewApprovalEvidence {
        gate: policy::ReviewGate,
        location_id: entities::LocationId,
        customer_id: entities::CustomerId,
        service_intent: entities::ServiceKind,
        channel: communication::Channel,
        purpose: communication::Purpose,
        message_ref: message::BodyRef,
    }

    impl fmt::Debug for ReviewApprovalEvidence {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("ReviewApprovalEvidence([REDACTED])")
        }
    }

    /// Opaque proof token for an outbound draft that may only enter an internal queue.
    ///
    /// No production issuer exists until an authenticated root of trust can accept exact review
    /// evidence. Serializable [`ReviewApprovalEvidence`] cannot construct this capability.
    pub struct QueueableContact {
        action: LegalContactAction,
        _authority: QueueContactAuthority,
    }

    struct QueueContactAuthority;

    impl QueueableContact {
        /// Action exposed by the opaque proof token.
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

    #[cfg(test)]
    mod coverage_convergence_tests {
        use chrono::TimeZone as _;
        use uuid::Uuid;

        use super::*;

        fn event(received_at: DateTime<Utc>) -> Event {
            Event::builder()
                .id(EventId::try_new("lead-7").unwrap())
                .idempotency_key(IdempotencyKey::try_new("dedupe-7").unwrap())
                .location_id(entities::LocationId::new(Uuid::from_u128(7)))
                .kind(EventKind::MissedCall)
                .received_at(received_at)
                .source_system(source::System::Telephony)
                .customer_match(identity::Match::Candidate {
                    customer_id: entities::CustomerId::new(Uuid::from_u128(8)),
                    confidence: identity::Confidence::High,
                })
                .service_intent(entities::ServiceKind::Boarding)
                .build()
        }

        fn consent(received_at: DateTime<Utc>, event: &Event) -> communication::ConsentEvidence {
            communication::ConsentEvidence::builder()
                .channel(communication::Channel::Sms)
                .purpose(communication::Purpose::TransactionalLeadResponse)
                .status(communication::ConsentStatus::Granted)
                .source(source::System::Crm)
                .subject(communication::Subject::SourceRecord(event.source_record()))
                .source_record(source::RecordRef::new(
                    source::System::Crm,
                    source::record::Id::try_new("consent-7").unwrap(),
                ))
                .source_schema_version(source::SchemaVersion::try_new("v1").unwrap())
                .effective_from(received_at - chrono::Duration::hours(1))
                .effective_until(received_at + chrono::Duration::hours(1))
                .build()
        }

        fn attempt(at: DateTime<Utc>) -> ContactAttempt {
            ContactAttempt::try_new(
                at,
                communication::Channel::Sms,
                communication::Purpose::TransactionalLeadResponse,
                AttemptOutcome::DraftedForReview,
                policy::ReviewGate::CustomerMessageApproval,
            )
            .unwrap()
            .with_message_ref(message::BodyRef::try_new("message-7").unwrap())
        }

        fn observation() -> ConversionObservation {
            ConversionObservation::try_reported_booking_observation(
                AttributionSource::MissedCall,
                None,
                Some(entities::reservation::Id::new(Uuid::from_u128(9))),
                Some(money::Money::usd(25_000).unwrap()),
            )
            .unwrap()
        }

        #[test]
        fn response_components_expose_exact_source_and_timing_evidence() {
            let received_at = Utc.with_ymd_and_hms(2026, 8, 18, 5, 0, 0).unwrap();
            let event = event(received_at);
            assert_eq!(
                format!("{:?}", event.idempotency_key()),
                "IdempotencyKey([REDACTED])"
            );
            assert_eq!(
                event.location_id(),
                entities::LocationId::new(Uuid::from_u128(7))
            );
            assert_eq!(event.received_at(), received_at);
            assert!(matches!(
                event.customer_match(),
                identity::Match::Candidate { .. }
            ));
            assert_eq!(event.service_intent(), entities::ServiceKind::Boarding);
            assert_eq!(event.source_system(), source::System::Telephony);

            assert_eq!(Minutes::try_new(0), Err(Error::ZeroMinutes));
            assert!(serde_json::from_value::<Minutes>(serde_json::json!(0)).is_err());
            let sla = ResponseSla::builder()
                .target(SlaTarget::FirstResponseWithinMinutes(
                    Minutes::try_new(5).unwrap(),
                ))
                .received_at(received_at)
                .due_at(received_at + chrono::Duration::minutes(5))
                .status(SlaStatus::Open)
                .build()
                .unwrap();
            assert_eq!(sla.received_at(), received_at);
            let decoded: ResponseSla =
                serde_json::from_value(serde_json::to_value(&sla).unwrap()).unwrap();
            assert_eq!(decoded.received_at(), received_at);
            let _default_sla_builder = ResponseSlaBuilder::default();
            assert_eq!(ResponseSla::builder().build(), Err(Error::MissingSlaField));
            assert_eq!(
                ResponseSla::try_new(
                    SlaTarget::FirstResponseWithinMinutes(Minutes::try_new(5).unwrap()),
                    received_at,
                    received_at + chrono::Duration::minutes(4),
                    SlaStatus::Open,
                ),
                Err(Error::SlaDueAtDoesNotMatchTarget)
            );

            let attempt = attempt(received_at);
            assert_eq!(attempt.attempted_at(), received_at);
            assert_eq!(attempt.channel(), communication::Channel::Sms);
            assert_eq!(
                attempt.purpose(),
                communication::Purpose::TransactionalLeadResponse
            );
            assert_eq!(
                attempt.message_ref(),
                Some(&message::BodyRef::try_new("message-7").unwrap())
            );
            assert_eq!(
                attempt.review_gate(),
                policy::ReviewGate::CustomerMessageApproval
            );
            assert_eq!(
                ConversionObservation::try_reported_booking_observation(
                    AttributionSource::Sms,
                    None,
                    None,
                    None,
                ),
                Err(Error::ConvertedLeadRequiresReservation)
            );
            assert!(!observation().can_support_value_claim());
        }

        #[test]
        fn response_packet_validation_rejects_missing_misaligned_and_untrusted_evidence() {
            let received_at = Utc.with_ymd_and_hms(2026, 8, 18, 5, 0, 0).unwrap();
            let sla = ResponseSla::try_new(
                SlaTarget::FirstResponseWithinMinutes(Minutes::try_new(5).unwrap()),
                received_at,
                received_at + chrono::Duration::minutes(5),
                SlaStatus::Open,
            )
            .unwrap();
            assert_eq!(
                ResponsePacket::builder().build(),
                Err(Error::MissingPacketField)
            );
            let base_event = event(received_at);
            let _default_packet_builder = ResponsePacketBuilder::default();
            assert_eq!(
                ResponsePacket::builder().event(base_event.clone()).build(),
                Err(Error::MissingPacketField)
            );
            assert_eq!(
                ResponsePacket::builder()
                    .event(base_event.clone())
                    .sla(sla.clone())
                    .build(),
                Err(Error::MissingPacketField)
            );
            assert_eq!(
                ResponsePacket::builder()
                    .event(base_event.clone())
                    .sla(sla.clone())
                    .consent(consent(received_at, &base_event))
                    .build(),
                Err(Error::MissingPacketField)
            );
            assert_eq!(
                ResponsePacket::builder()
                    .event(base_event.clone())
                    .sla(sla.clone())
                    .consent(consent(received_at, &base_event))
                    .attribution(observation())
                    .build(),
                Err(Error::MissingContactAttempt)
            );
            assert_eq!(
                ResponsePacket::builder()
                    .event(base_event.clone())
                    .sla(sla.clone())
                    .consent(consent(received_at, &base_event))
                    .attempts(vec![attempt(received_at)])
                    .build(),
                Err(Error::MissingPacketField)
            );
            assert_eq!(
                ResponsePacket::try_new(
                    event(received_at),
                    ResponseSla::try_new(
                        SlaTarget::FirstResponseWithinMinutes(Minutes::try_new(5).unwrap()),
                        received_at + chrono::Duration::minutes(1),
                        received_at + chrono::Duration::minutes(6),
                        SlaStatus::Open,
                    )
                    .unwrap(),
                    consent(received_at, &event(received_at)),
                    vec![attempt(received_at)],
                    observation(),
                ),
                Err(Error::SlaReceiptDoesNotMatchEvent)
            );
            assert_eq!(
                ResponsePacket::try_new(
                    event(received_at),
                    sla.clone(),
                    consent(received_at, &event(received_at)),
                    Vec::new(),
                    observation(),
                ),
                Err(Error::MissingContactAttempt)
            );
            assert_eq!(
                ResponsePacket::try_new(
                    event(received_at),
                    sla.clone(),
                    consent(received_at, &event(received_at)),
                    vec![attempt(received_at - chrono::Duration::seconds(1))],
                    observation(),
                ),
                Err(Error::ContactAttemptBeforeReceipt)
            );
            assert_eq!(
                ResponsePacket::try_new(
                    event(received_at),
                    sla.clone(),
                    consent(received_at, &event(received_at)),
                    vec![
                        attempt(received_at + chrono::Duration::minutes(2)),
                        attempt(received_at + chrono::Duration::minutes(1)),
                    ],
                    observation(),
                ),
                Err(Error::ContactAttemptsOutOfOrder)
            );
            assert_eq!(
                ResponsePacket::try_new(
                    event(received_at),
                    sla,
                    consent(received_at, &event(received_at)),
                    vec![attempt(received_at)],
                    observation(),
                ),
                Err(Error::ConsentDoesNotCoverAttempt)
            );
        }

        #[test]
        fn opaque_accepted_consent_exercises_shared_packet_validation_without_enabling_send() {
            let received_at = Utc.with_ymd_and_hms(2026, 8, 18, 5, 0, 0).unwrap();
            let event = event(received_at);
            let accepted =
                communication::issue_accepted_consent(consent(received_at, &event)).unwrap();
            let packet = ResponsePacket::try_new_with_accepted_consent(
                event.clone(),
                ResponseSla::try_new(
                    SlaTarget::FirstResponseWithinMinutes(Minutes::try_new(5).unwrap()),
                    received_at,
                    received_at + chrono::Duration::minutes(5),
                    SlaStatus::Open,
                )
                .unwrap(),
                accepted,
                vec![attempt(received_at)],
                observation(),
            )
            .unwrap();

            assert_eq!(packet.event().source_record(), event.source_record());
            assert!(packet.requires_customer_message_approval());
            assert!(!packet.consent_allows_response(
                communication::Channel::Sms,
                communication::Purpose::TransactionalLeadResponse,
            ));
        }

        #[test]
        fn internally_constructed_packet_and_queue_capability_keep_live_send_disabled() {
            let received_at = Utc.with_ymd_and_hms(2026, 8, 18, 5, 0, 0).unwrap();
            let event = event(received_at);
            let packet = ResponsePacket {
                event: event.clone(),
                sla: ResponseSla::try_new(
                    SlaTarget::FirstResponseWithinMinutes(Minutes::try_new(5).unwrap()),
                    received_at,
                    received_at + chrono::Duration::minutes(5),
                    SlaStatus::Open,
                )
                .unwrap(),
                consent: consent(received_at, &event),
                attempts: vec![attempt(received_at)],
                attribution: observation(),
            };
            let serialized = serde_json::to_value(&packet).unwrap();
            assert!(serde_json::from_value::<ResponsePacket>(serialized).is_err());
            assert_eq!(packet.event().location_id(), event.location_id());
            assert_eq!(packet.attempts().len(), 1);
            assert!(packet.requires_customer_message_approval());
            assert!(!packet.consent_allows_response(
                communication::Channel::Sms,
                communication::Purpose::TransactionalLeadResponse,
            ));
            let queueable = QueueableContact {
                action: LegalContactAction::QueueOnly,
                _authority: QueueContactAuthority,
            };
            assert_eq!(queueable.action(), LegalContactAction::QueueOnly);
            assert!(queueable.live_send_is_unavailable());
        }
    }
}
