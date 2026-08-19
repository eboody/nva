//! Fixture-safe Gingr webhook parsing and acknowledgement surfaces.
//!
//! Webhooks are parsed into a quarantined envelope first. Verification is explicit,
//! uses a caller-provided secret, and failure maps to an acknowledgement without
//! mutating provider state or sending customer messages.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use gingr::{response, webhook};
//!
//! let raw = r#"{
//!   "webhook_type": "animal_edited",
//!   "entity_id": 812,
//!   "entity_type": "animal",
//!   "entity_data": {"name": "Miso"}
//! }"#;
//! let envelope = webhook::Envelope::from_json(raw)?;
//! assert_eq!(envelope.event_type_input(), Some("animal_edited"));
//!
//! let missing_signature = envelope.verify(&webhook::SignatureKey::from_secret("fixture-only"));
//! assert!(matches!(
//!     missing_signature,
//!     Err(webhook::VerificationError::MissingField { field: "signature" })
//! ));
//! assert_eq!(
//!     webhook::Ack::RejectedPermanently.http_status(),
//!     response::HttpStatus::FORBIDDEN
//! );
//! # Ok(())
//! # }
//! ```

use crate::response;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
/// Gingr webhook event names normalized from provider strings while retaining unknown events.
pub enum EventType {
    /// Gingr event fired when a reservation checks in.
    CheckIn,
    /// Gingr event fired when a reservation checks out.
    CheckOut,
    /// Gingr in-progress check-in event.
    CheckingIn,
    /// Gingr in-progress check-out event.
    CheckingOut,
    /// Gingr event for outbound email activity.
    EmailSent,
    /// Gingr event for a newly created owner record.
    OwnerCreated,
    /// Gingr event for changes to an owner record.
    OwnerEdited,
    /// Gingr event for a newly created animal record.
    AnimalCreated,
    /// Gingr event for changes to an animal record.
    AnimalEdited,
    /// Gingr event for a newly created incident.
    IncidentCreated,
    /// Gingr event for changes to an incident.
    IncidentEdited,
    /// Gingr event for a newly created lead.
    LeadCreated,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    #[strum(default)]
    Unknown(String),
}

impl EventType {
    /// Returns the exact Gingr token used when acknowledging or auditing provider events.
    pub fn as_provider_str(&self) -> &str {
        match self {
            Self::Unknown(raw) => raw.as_str(),
            known => known.as_ref(),
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_provider_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
/// Gingr webhook entity classes normalized from provider strings while retaining unknown entities.
pub enum EntityType {
    /// Webhook entity is a Gingr reservation.
    Reservation,
    /// Provider value refers to a Gingr owner/customer.
    Owner,
    /// Provider value refers to a Gingr animal/pet.
    Animal,
    /// Provider value refers to a Gingr incident.
    Incident,
    /// Provider value refers to a Gingr lead.
    Lead,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    #[strum(default)]
    Unknown(String),
}

impl EntityType {
    /// Returns the exact Gingr token used when acknowledging or auditing provider events.
    pub fn as_provider_str(&self) -> &str {
        match self {
            Self::Unknown(raw) => raw.as_str(),
            known => known.as_ref(),
        }
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_provider_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Normalized Gingr entity identifier preserved as text across numeric and string webhook inputs.
pub struct EntityId(String);

impl EntityId {
    /// Returns the normalized provider or storage string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, PartialEq)]
/// Raw Gingr webhook envelope before signature verification and required-field promotion.
pub struct Envelope {
    wire: WireEnvelope,
}

impl fmt::Debug for Envelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Envelope")
            .field("webhook_type", &self.wire.webhook_type)
            .field(
                "entity_id_type",
                &self.wire.entity_id.as_ref().map(json_type_name),
            )
            .field("entity_type", &self.wire.entity_type)
            .field("signature_present", &self.wire.signature.is_some())
            .finish_non_exhaustive()
    }
}

impl Envelope {
    /// Returns the raw event type field supplied by Gingr, if present.
    pub fn event_type_input(&self) -> Option<&str> {
        self.wire.webhook_type.as_deref()
    }

    /// Returns the raw entity type field supplied by Gingr, if present.
    pub fn entity_type_input(&self) -> Option<&str> {
        self.wire.entity_type.as_deref()
    }

    /// Returns the signature value supplied with the webhook payload, if present.
    pub fn signature_input(&self) -> Option<&str> {
        self.wire.signature.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Webhook payload that passed signature validation and required entity/event checks.
pub struct Verified {
    event_type: EventType,
    entity_id: EntityId,
    entity_type: EntityType,
    payload: Payload,
}

impl Verified {
    /// Returns the normalized Gingr event type for a verified webhook.
    pub fn event_type(&self) -> EventType {
        self.event_type.clone()
    }

    /// Returns the provider entity identifier from the verified webhook.
    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    /// Returns the normalized Gingr entity type for a verified webhook.
    pub fn entity_type(&self) -> EntityType {
        self.entity_type.clone()
    }

    /// Returns the verified provider payload for downstream mapping.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

#[derive(Clone, PartialEq)]
/// Provider-specific webhook payload body retained for downstream DTO mapping.
pub struct Payload {
    wire: WireEnvelope,
}

impl fmt::Debug for Payload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Payload")
            .field("provider_url", &self.wire.webhook_url)
            .field("has_entity_data", &!self.wire.entity_data.is_null())
            .field("has_email_data", &self.wire.email_data.is_some())
            .field(
                "recipient_count",
                &self.wire.recipients.as_ref().map(Vec::len),
            )
            .finish_non_exhaustive()
    }
}

impl Payload {
    /// Returns the Gingr entity payload object supplied with the webhook for downstream mapping.
    pub fn entity_data(&self) -> &serde_json::Value {
        &self.wire.entity_data
    }

    /// Returns Gingr email metadata when the event includes it.
    pub fn email_data(&self) -> Option<&serde_json::Value> {
        self.wire.email_data.as_ref()
    }

    /// Returns the provider recipient list for email-related webhook events.
    pub fn recipients(&self) -> Option<&Vec<serde_json::Value>> {
        self.wire.recipients.as_ref()
    }

    /// Returns Gingr URL metadata included in the payload, if supplied.
    pub fn provider_url(&self) -> Option<&str> {
        self.wire.webhook_url.as_deref()
    }
}

#[derive(Clone, PartialEq, serde::Deserialize)]
struct WireEnvelope {
    webhook_url: Option<String>,
    webhook_type: Option<String>,
    entity_id: Option<serde_json::Value>,
    entity_type: Option<String>,
    signature: Option<String>,
    #[serde(default)]
    entity_data: serde_json::Value,
    email_data: Option<serde_json::Value>,
    recipients: Option<Vec<serde_json::Value>>,
    #[serde(flatten)]
    unknown: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
/// Reasons a Gingr webhook cannot be trusted or promoted.
pub enum VerificationError {
    #[error("Gingr webhook is missing required field {field}")]
    /// Webhook omitted a required provider field.
    MissingField {
        /// Provider field required before this mapping can create a source-backed candidate.
        field: &'static str,
    },
    #[error("unsupported Gingr webhook entity_id representation: {observed_type}")]
    /// Webhook used an entity_id shape this integration cannot normalize.
    UnsupportedEntityId {
        /// Observed type attached to this Gingr error or DTO.
        observed_type: String,
    },
    #[error("malformed Gingr webhook signature: {reason}")]
    /// Webhook signature could not be parsed before comparison.
    MalformedSignature {
        /// Reason the supplied Gingr signature could not be decoded before comparison.
        reason: String,
    },
    #[error("Gingr webhook signature mismatch")]
    /// Computed signature did not match the value supplied by Gingr.
    SignatureMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// HTTP acknowledgement categories returned to Gingr after webhook handling.
pub enum Ack {
    /// Webhook was accepted and no retry is requested.
    Processed,
    /// Webhook failed validation in a way Gingr should not retry.
    RejectedPermanently,
    /// Webhook processing failed transiently and may be retried.
    RetryableFailure,
    /// Propagates a retryable downstream HTTP status while acknowledging Gingr semantics.
    RetryableStatus(response::HttpStatus),
}

impl Ack {}

fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(number) if number.is_f64() => "decimal number",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
