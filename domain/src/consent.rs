//! Consent purpose and evidence contracts for fail-closed communication review.
//!
//! Consent evidence is an observation, not authority. It may be incomplete while source discovery is
//! underway. Every public [`crate::consent::ConsentEvidence`] permission predicate fails closed. Only
//! opaque, non-serializable [`crate::consent::AcceptedConsent`] can apply exact subject,
//! source/version, interval, and unsuperseded-revocation matching; no production issuer is
//! currently exposed.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{entities, message, source};

/// Canonical message/contact channel used for communication evidence or draft outreach.
pub use message::Channel;

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
    /// Source evidence reports a granted label; only [`AcceptedConsent`] can grant use.
    Granted,
    /// Consent evidence is missing.
    Missing,
    /// Customer opted out or suppressed the channel/purpose.
    OptedOut,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Exact subject to which source consent evidence applies.
pub enum Subject {
    /// Consent applies to one owned customer identity.
    Customer(entities::CustomerId),
    /// Consent applies to one source record before an owned customer identity is accepted.
    SourceRecord(source::RecordRef),
}

impl std::fmt::Debug for Subject {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Subject([REDACTED])")
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
/// Source-backed consent evidence for one subject, channel, purpose, and effective interval.
///
/// Optional binding fields preserve honest incomplete observations during source discovery. Such
/// observations remain serializable but cannot derive communication permission.
pub struct ConsentEvidence {
    channel: Channel,
    purpose: Purpose,
    status: ConsentStatus,
    source: source::System,
    subject: Option<Subject>,
    source_record: Option<source::RecordRef>,
    source_schema_version: Option<source::SchemaVersion>,
    effective_from: Option<DateTime<Utc>>,
    effective_until: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
    superseded_by: Option<source::RecordRef>,
}

impl std::fmt::Debug for ConsentEvidence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ConsentEvidence([REDACTED])")
    }
}

/// Opaque accepted consent promoted by a trusted source adapter.
///
/// The contained history remains serializable through [`ConsentEvidence`], but accepted consent is
/// intentionally neither serializable nor caller-constructible. Production issuance is absent until
/// a source-authenticated consent adapter owns this trust transition.
pub struct AcceptedConsent {
    evidence: ConsentEvidence,
}

impl AcceptedConsent {
    /// Checks the exact subject, purpose, channel, provenance, and effective lifecycle.
    pub fn permits_customer(
        &self,
        customer_id: entities::CustomerId,
        channel: Channel,
        purpose: Purpose,
        as_of: DateTime<Utc>,
    ) -> bool {
        self.evidence
            .matches_customer(customer_id, channel, purpose, as_of)
    }

    /// Returns the serializable historical evidence retained by this accepted fact.
    pub const fn evidence(&self) -> &ConsentEvidence {
        &self.evidence
    }
}

#[cfg(test)]
pub(crate) fn issue_accepted_consent(evidence: ConsentEvidence) -> Option<AcceptedConsent> {
    let source_record = evidence.source_record.as_ref()?;
    let _source_schema_version = evidence.source_schema_version.as_ref()?;
    let effective_from = evidence.effective_from?;
    let interval_is_forward = evidence
        .effective_until
        .is_none_or(|effective_until| effective_until > effective_from);
    let revocation_is_chronological = evidence
        .revoked_at
        .is_none_or(|revoked_at| revoked_at >= effective_from);

    (evidence.subject.is_some()
        && source_record.system() == evidence.source
        && interval_is_forward
        && revocation_is_chronological)
        .then_some(AcceptedConsent { evidence })
}

impl ConsentEvidence {
    /// Returns whether this consent evidence reports a grant.
    pub const fn is_granted(&self) -> bool {
        matches!(self.status, ConsentStatus::Granted)
    }

    /// Returns false because serializable evidence cannot grant a channel/purpose use.
    pub const fn permits(&self, _channel: Channel, _purpose: Purpose) -> bool {
        false
    }

    fn reports_grant_for(&self, channel: Channel, purpose: Purpose) -> bool {
        self.channel == channel && self.purpose == purpose && self.is_granted()
    }

    fn is_current_at(&self, as_of: DateTime<Utc>) -> bool {
        self.effective_from.is_some_and(|start| start <= as_of)
            && self.effective_until.is_none_or(|end| as_of < end)
            && self.revoked_at.is_none_or(|revoked| as_of < revoked)
            && self.superseded_by.is_none()
    }

    fn has_complete_source_binding(&self) -> bool {
        self.source_record
            .as_ref()
            .is_some_and(|record| record.system() == self.source)
            && self.source_schema_version.is_some()
    }

    /// Returns false because serializable evidence cannot mint customer-contact permission.
    ///
    /// Exact matching is promoted only behind [`AcceptedConsent`].
    pub fn permits_customer(
        &self,
        _customer_id: entities::CustomerId,
        _channel: Channel,
        _purpose: Purpose,
        _as_of: DateTime<Utc>,
    ) -> bool {
        false
    }

    fn matches_customer(
        &self,
        customer_id: entities::CustomerId,
        channel: Channel,
        purpose: Purpose,
        as_of: DateTime<Utc>,
    ) -> bool {
        self.subject == Some(Subject::Customer(customer_id))
            && self.reports_grant_for(channel, purpose)
            && self.has_complete_source_binding()
            && self.is_current_at(as_of)
    }

    /// Returns false because serializable evidence cannot mint source-record contact permission.
    ///
    /// Production promotion requires an unavailable source-authenticated accepted-consent issuer.
    pub fn permits_source_record(
        &self,
        _subject: &source::RecordRef,
        _channel: Channel,
        _purpose: Purpose,
        _as_of: DateTime<Utc>,
    ) -> bool {
        false
    }

    /// Channel covered by this evidence.
    pub const fn channel(&self) -> Channel {
        self.channel
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::*;

    #[test]
    fn consent_subject_and_evidence_debug_redact_customer_identity() {
        let private_customer = entities::CustomerId::new(
            Uuid::parse_str("86753090-1234-4567-89ab-cdef01234567").unwrap(),
        );
        let subject = Subject::Customer(private_customer);
        let evidence = ConsentEvidence::builder()
            .channel(Channel::Email)
            .purpose(Purpose::MarketingRetention)
            .status(ConsentStatus::Granted)
            .source(source::System::Crm)
            .subject(subject.clone())
            .build();

        assert_eq!(format!("{subject:?}"), "Subject([REDACTED])");
        assert_eq!(format!("{evidence:?}"), "ConsentEvidence([REDACTED])");
        assert!(
            !format!("{subject:?}{evidence:?}").contains("86753090-1234-4567-89ab-cdef01234567")
        );
    }

    #[test]
    fn accepted_consent_remains_exact_subject_and_effective_time_bound() {
        let customer = entities::CustomerId::new(uuid::Uuid::from_u128(1));
        let evidence = ConsentEvidence::builder()
            .channel(Channel::Email)
            .purpose(Purpose::MarketingRetention)
            .status(ConsentStatus::Granted)
            .source(source::System::Crm)
            .subject(Subject::Customer(customer))
            .source_record(source::RecordRef::new(
                source::System::Crm,
                source::record::Id::try_new("consent-1").unwrap(),
            ))
            .source_schema_version(source::SchemaVersion::try_new("consent-v1").unwrap())
            .effective_from(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap())
            .effective_until(Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap())
            .build();
        let accepted = issue_accepted_consent(evidence).unwrap();
        let as_of = Utc.with_ymd_and_hms(2026, 8, 15, 0, 0, 0).unwrap();

        assert!(accepted.permits_customer(
            customer,
            Channel::Email,
            Purpose::MarketingRetention,
            as_of,
        ));
        assert!(!accepted.permits_customer(
            entities::CustomerId::new(Uuid::from_u128(2)),
            Channel::Email,
            Purpose::MarketingRetention,
            as_of,
        ));
        assert!(!accepted.permits_customer(
            customer,
            Channel::Email,
            Purpose::MarketingRetention,
            Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap(),
        ));
    }
}
