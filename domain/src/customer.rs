//! Customer identity and contact values used by reservation and messaging workflows.
//!
//! These newtypes promote portal/import strings into validated customer identifiers and
//! contact values before inbox drafting, reservation triage, or follow-up can use them.
//! They do not prove identity or consent by themselves; adapters must still attach provenance and
//! any required approval gates before live customer communication.

use chrono::{DateTime, Utc};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{access, consent as communication, entities, identity, policy};

/// Reviewed customer intelligence, structured notes, and segment-membership contracts.
///
/// Canonical owner for CRM/customer-intelligence concepts previously introduced by
/// `strategic_ai_ops::crm`. Operations-only facts must not become marketing eligibility without
/// explicit allowed-use, visibility, consent, and review evidence.
pub mod intelligence {
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
            AsRef,
            Serialize,
            Deserialize
        )
    )]
    /// Version of a reviewed customer segment definition.
    pub struct SegmentVersion(String);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    /// Half-open interval during which CRM evidence or segment membership may be used.
    pub struct EffectiveInterval {
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    }

    impl EffectiveInterval {
        /// Creates an interval only when the optional end follows the start.
        pub fn try_new(start: DateTime<Utc>, end: Option<DateTime<Utc>>) -> Result<Self> {
            if end.is_some_and(|end| end <= start) {
                return Err(Error::EffectiveIntervalEndMustFollowStart);
            }
            Ok(Self { start, end })
        }

        /// Returns whether the supplied instant falls inside the interval.
        pub fn contains(&self, at: DateTime<Utc>) -> bool {
            at >= self.start && self.end.is_none_or(|end| at < end)
        }
    }

    impl<'de> Deserialize<'de> for EffectiveInterval {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawEffectiveInterval {
                start: DateTime<Utc>,
                end: Option<DateTime<Utc>>,
            }

            let raw = RawEffectiveInterval::deserialize(deserializer)?;
            Self::try_new(raw.start, raw.end).map_err(serde::de::Error::custom)
        }
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
        reviewed_by: Option<access::ActorId>,
        effective_interval: EffectiveInterval,
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

        /// Customer identity this note belongs to.
        pub const fn customer_id(&self) -> entities::CustomerId {
            self.customer_id
        }

        /// Pet identity this note belongs to, when the fact is pet-specific.
        pub const fn pet_id(&self) -> Option<entities::PetId> {
            self.pet_id
        }

        /// Visibility scope approved for this note.
        pub const fn visibility(&self) -> access::VisibilityScope {
            self.visibility
        }

        /// Review lifecycle state.
        pub const fn review_state(&self) -> ReviewState {
            self.review_state
        }

        /// Reviewer or role account that accepted the note, if reviewed.
        pub const fn reviewed_by(&self) -> Option<&access::ActorId> {
            self.reviewed_by.as_ref()
        }

        /// Effective interval for this note.
        pub const fn effective_interval(&self) -> &EffectiveInterval {
            &self.effective_interval
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Accepted CRM note proof after review, scope, use, and interval checks.
    pub struct AcceptedNote {
        note: StructuredNote,
        accepted_for: access::AllowedUse,
    }

    impl AcceptedNote {
        /// Promotes a reviewed note only when it is accepted, current, scoped, and supports the use.
        pub fn try_from_reviewed(
            note: StructuredNote,
            as_of: DateTime<Utc>,
            allowed_use: access::AllowedUse,
        ) -> Result<Self> {
            if note.review_state() != ReviewState::Accepted || note.reviewed_by().is_none() {
                return Err(Error::NoteNotAccepted);
            }
            if !note.effective_interval().contains(as_of) {
                return Err(Error::EvidenceExpired);
            }
            if !note.can_support(allowed_use) {
                return Err(Error::AllowedUseMismatch);
            }
            match allowed_use {
                access::AllowedUse::MarketingCampaign
                    if note.visibility() != access::VisibilityScope::MarketingEligible =>
                {
                    Err(Error::VisibilityUseMismatch)
                }
                _ => Ok(Self {
                    note,
                    accepted_for: allowed_use,
                }),
            }
        }

        /// Customer identity this accepted note belongs to.
        pub const fn customer_id(&self) -> entities::CustomerId {
            self.note.customer_id()
        }

        /// Pet identity this accepted note belongs to, when pet-specific.
        pub const fn pet_id(&self) -> Option<entities::PetId> {
            self.note.pet_id()
        }

        /// Returns whether this accepted note can support the use at the supplied time.
        pub fn supports(&self, allowed_use: access::AllowedUse, as_of: DateTime<Utc>) -> bool {
            self.accepted_for == allowed_use && self.note.effective_interval().contains(as_of)
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
    /// Versioned segment definition with declared visibility, use, and review boundary.
    pub struct SegmentDefinition {
        segment: Segment,
        version: SegmentVersion,
        allowed_use: access::AllowedUse,
        visibility: access::VisibilityScope,
        review_gate: policy::ReviewGate,
    }

    impl SegmentDefinition {
        /// Segment label this definition produces.
        pub const fn segment(&self) -> Segment {
            self.segment
        }

        /// Definition version.
        pub const fn version(&self) -> &SegmentVersion {
            &self.version
        }

        /// Declared use for this definition.
        pub const fn allowed_use(&self) -> access::AllowedUse {
            self.allowed_use
        }

        /// Visibility required for evidence behind this definition.
        pub const fn visibility(&self) -> access::VisibilityScope {
            self.visibility
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    /// Reviewed segment membership whose use permission must be derived from evidence and consent.
    pub struct SegmentMembership {
        customer_id: entities::CustomerId,
        definition: SegmentDefinition,
        basis: Vec<SegmentBasis>,
        effective_interval: EffectiveInterval,
        evidence: Vec<AcceptedNote>,
    }

    impl SegmentMembership {
        /// Creates segment membership only when evidence is non-empty, same-customer, and scope-compatible.
        pub fn try_new(
            customer_id: entities::CustomerId,
            definition: SegmentDefinition,
            basis: Vec<SegmentBasis>,
            effective_interval: EffectiveInterval,
            evidence: Vec<AcceptedNote>,
        ) -> Result<Self> {
            if basis.is_empty() || evidence.is_empty() {
                return Err(Error::MissingEvidence);
            }
            for note in &evidence {
                if note.customer_id() != customer_id {
                    return Err(Error::EvidenceCustomerMismatch);
                }
                if !note.supports(definition.allowed_use(), effective_interval.start) {
                    return Err(Error::AllowedUseMismatch);
                }
            }
            Ok(Self {
                customer_id,
                definition,
                basis,
                effective_interval,
                evidence,
            })
        }

        /// Customer identity this membership belongs to.
        pub const fn customer_id(&self) -> entities::CustomerId {
            self.customer_id
        }

        /// Segment label this membership carries.
        pub const fn segment(&self) -> Segment {
            self.definition.segment()
        }

        /// Segment definition used to produce this membership.
        pub const fn definition(&self) -> &SegmentDefinition {
            &self.definition
        }

        /// Accepted note evidence backing this membership.
        pub fn evidence(&self) -> &[AcceptedNote] {
            &self.evidence
        }

        /// Returns whether membership is current for a supplied instant.
        pub fn is_current_at(&self, as_of: DateTime<Utc>) -> bool {
            self.effective_interval.contains(as_of)
        }
    }

    impl<'de> Deserialize<'de> for SegmentMembership {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RawSegmentMembership {
                customer_id: entities::CustomerId,
                definition: SegmentDefinition,
                basis: Vec<SegmentBasis>,
                effective_interval: EffectiveInterval,
                evidence: Vec<AcceptedNote>,
            }
            let raw = RawSegmentMembership::deserialize(deserializer)?;
            Self::try_new(
                raw.customer_id,
                raw.definition,
                raw.basis,
                raw.effective_interval,
                raw.evidence,
            )
            .map_err(serde::de::Error::custom)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Proof that a current segment membership may be used for a declared marketing action.
    pub struct MarketingUsePermission {
        customer_id: entities::CustomerId,
        segment: Segment,
        allowed_use: access::AllowedUse,
        consent: communication::ConsentEvidence,
    }

    impl MarketingUsePermission {
        /// Derives marketing-use permission from current membership and exact consent evidence.
        pub fn try_from_membership(
            membership: &SegmentMembership,
            consent: Option<communication::ConsentEvidence>,
            allowed_use: access::AllowedUse,
            as_of: DateTime<Utc>,
        ) -> Result<Self> {
            if allowed_use != access::AllowedUse::MarketingCampaign {
                return Err(Error::AllowedUseMismatch);
            }
            if membership.definition().allowed_use() != allowed_use
                || !membership.is_current_at(as_of)
            {
                return Err(Error::AllowedUseMismatch);
            }
            let consent = consent.ok_or(Error::MissingMarketingConsentEvidence)?;
            if !consent.permits(
                communication::Channel::Email,
                communication::Purpose::MarketingRetention,
            ) {
                return Err(Error::MissingMarketingConsentEvidence);
            }
            Ok(Self {
                customer_id: membership.customer_id(),
                segment: membership.segment(),
                allowed_use,
                consent,
            })
        }

        /// Customer identity authorized for this marketing use.
        pub const fn customer_id(&self) -> entities::CustomerId {
            self.customer_id
        }

        /// Segment authorized for this marketing use.
        pub const fn segment(&self) -> Segment {
            self.segment
        }

        /// Declared allowed use proven by this permission.
        pub const fn allowed_use(&self) -> access::AllowedUse {
            self.allowed_use
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    /// Customer-intelligence validation failures.
    pub enum Error {
        #[error("effective interval end must follow start")]
        /// Effective interval was empty or reversed.
        EffectiveIntervalEndMustFollowStart,
        #[error("CRM note must be accepted by a reviewer before promotion")]
        /// Candidate, rejected, superseded, or unreviewed notes cannot become accepted evidence.
        NoteNotAccepted,
        #[error("CRM evidence is not effective at the requested time")]
        /// Evidence or membership interval is expired or not yet active.
        EvidenceExpired,
        #[error("CRM evidence allowed use does not match the requested use")]
        /// Evidence allowed-use scope does not cover the requested application.
        AllowedUseMismatch,
        #[error("CRM visibility scope does not allow the requested use")]
        /// Visibility scope cannot support the requested application.
        VisibilityUseMismatch,
        #[error("segment membership requires non-empty evidence and basis")]
        /// Segment membership cannot be asserted without basis and accepted notes.
        MissingEvidence,
        #[error("segment evidence belongs to a different customer")]
        /// Segment evidence customer did not match membership customer.
        EvidenceCustomerMismatch,
        #[error("marketing use requires explicit source-backed marketing consent evidence")]
        /// Marketing consent was missing or did not cover marketing retention.
        MissingMarketingConsentEvidence,
    }

    /// Result type returned by customer-intelligence constructors.
    pub type Result<T> = std::result::Result<T, Error>;
}

/// Customer display name as staff, portal records, and customer-facing drafts should show it.
///
/// The value is trimmed and bounded so generated messages, manager briefings, and search indexes
/// can carry a stable customer label without accepting blank source data.
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
pub struct Name(String);

/// Customer email address captured from a portal, staff entry, import, or message source.
///
/// This type only enforces the storage/display envelope; workflows must still respect channel
/// consent, approval state, and resort policy before sending outbound email.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 254),
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
pub struct Email(String);

/// Customer phone number text used for call, SMS, and staff-note correlation.
///
/// The type preserves a normalized non-empty contact string for source-derived records; it is not
/// a permission to text or call without the message-policy and review gates required upstream.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 40),
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
pub struct Phone(String);
