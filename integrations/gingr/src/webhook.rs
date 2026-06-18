//! Fixture-safe Gingr webhook parsing and acknowledgement contracts.
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
use hmac::{Hmac, Mac};
use secrecy::SecretString;
use sha2::Sha256;
use std::fmt;
use subtle::ConstantTimeEq;

pub type VerificationResult<T> = core::result::Result<T, VerificationError>;
pub type ParseResult<T> = core::result::Result<T, ParseError>;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct SignatureKey(SecretString);

impl SignatureKey {
    pub fn from_secret(raw: impl Into<String>) -> Self {
        Self(SecretString::new(raw.into()))
    }

    fn expose_for_verification(&self) -> &str {
        use secrecy::ExposeSecret;
        self.0.expose_secret()
    }
}

impl fmt::Debug for SignatureKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SignatureKey(<redacted>)")
    }
}

impl fmt::Display for SignatureKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    CheckIn,
    CheckOut,
    CheckingIn,
    CheckingOut,
    EmailSent,
    OwnerCreated,
    OwnerEdited,
    AnimalCreated,
    AnimalEdited,
    IncidentCreated,
    IncidentEdited,
    LeadCreated,
    Unknown(String),
}

impl EventType {
    pub fn parse(raw: impl AsRef<str>) -> Self {
        match raw.as_ref() {
            "check_in" => Self::CheckIn,
            "check_out" => Self::CheckOut,
            "checking_in" => Self::CheckingIn,
            "checking_out" => Self::CheckingOut,
            "email_sent" => Self::EmailSent,
            "owner_created" => Self::OwnerCreated,
            "owner_edited" => Self::OwnerEdited,
            "animal_created" => Self::AnimalCreated,
            "animal_edited" => Self::AnimalEdited,
            "incident_created" => Self::IncidentCreated,
            "incident_edited" => Self::IncidentEdited,
            "lead_created" => Self::LeadCreated,
            other => Self::Unknown(other.to_owned()),
        }
    }

    pub fn as_provider_str(&self) -> &str {
        match self {
            Self::CheckIn => "check_in",
            Self::CheckOut => "check_out",
            Self::CheckingIn => "checking_in",
            Self::CheckingOut => "checking_out",
            Self::EmailSent => "email_sent",
            Self::OwnerCreated => "owner_created",
            Self::OwnerEdited => "owner_edited",
            Self::AnimalCreated => "animal_created",
            Self::AnimalEdited => "animal_edited",
            Self::IncidentCreated => "incident_created",
            Self::IncidentEdited => "incident_edited",
            Self::LeadCreated => "lead_created",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityType {
    Reservation,
    Owner,
    Animal,
    Incident,
    Lead,
    Unknown(String),
}

impl EntityType {
    pub fn parse(raw: impl AsRef<str>) -> Self {
        match raw.as_ref() {
            "reservation" => Self::Reservation,
            "owner" => Self::Owner,
            "animal" => Self::Animal,
            "incident" => Self::Incident,
            "lead" => Self::Lead,
            other => Self::Unknown(other.to_owned()),
        }
    }

    pub fn as_provider_str(&self) -> &str {
        match self {
            Self::Reservation => "reservation",
            Self::Owner => "owner",
            Self::Animal => "animal",
            Self::Incident => "incident",
            Self::Lead => "lead",
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityId(String);

impl EntityId {
    fn normalize(value: &serde_json::Value) -> VerificationResult<Self> {
        match value {
            serde_json::Value::String(raw) if !raw.is_empty() => Ok(Self(raw.clone())),
            serde_json::Value::Number(number) if number.is_i64() || number.is_u64() => {
                Ok(Self(number.to_string()))
            }
            other => Err(VerificationError::UnsupportedEntityId {
                observed_type: json_type_name(other).to_owned(),
            }),
        }
    }

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
    pub fn from_json(raw: impl AsRef<str>) -> ParseResult<Self> {
        let wire = serde_json::from_str(raw.as_ref())?;
        Ok(Self { wire })
    }

    pub fn event_type_input(&self) -> Option<&str> {
        self.wire.webhook_type.as_deref()
    }

    pub fn entity_type_input(&self) -> Option<&str> {
        self.wire.entity_type.as_deref()
    }

    pub fn signature_input(&self) -> Option<&str> {
        self.wire.signature.as_deref()
    }

    pub fn verify(self, key: &SignatureKey) -> VerificationResult<Verified> {
        let webhook_type =
            self.wire
                .webhook_type
                .as_deref()
                .ok_or(VerificationError::MissingField {
                    field: "webhook_type",
                })?;
        let entity_id_value = self
            .wire
            .entity_id
            .as_ref()
            .ok_or(VerificationError::MissingField { field: "entity_id" })?;
        let entity_id = EntityId::normalize(entity_id_value)?;
        let entity_type =
            self.wire
                .entity_type
                .as_deref()
                .ok_or(VerificationError::MissingField {
                    field: "entity_type",
                })?;
        let supplied_signature = self
            .wire
            .signature
            .as_deref()
            .ok_or(VerificationError::MissingField { field: "signature" })?;

        verify_signature(
            key,
            webhook_type,
            entity_id.as_str(),
            entity_type,
            supplied_signature,
        )?;

        Ok(Verified {
            event_type: EventType::parse(webhook_type),
            entity_id,
            entity_type: EntityType::parse(entity_type),
            payload: Payload { wire: self.wire },
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Verified {
    event_type: EventType,
    entity_id: EntityId,
    entity_type: EntityType,
    payload: Payload,
}

impl Verified {
    pub fn event_type(&self) -> EventType {
        self.event_type.clone()
    }

    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    pub fn entity_type(&self) -> EntityType {
        self.entity_type.clone()
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

#[derive(Clone, PartialEq)]
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
    pub fn entity_data(&self) -> &serde_json::Value {
        &self.wire.entity_data
    }

    pub fn email_data(&self) -> Option<&serde_json::Value> {
        self.wire.email_data.as_ref()
    }

    pub fn recipients(&self) -> Option<&Vec<serde_json::Value>> {
        self.wire.recipients.as_ref()
    }

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

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid Gingr webhook JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VerificationError {
    #[error("Gingr webhook is missing required field {field}")]
    MissingField { field: &'static str },
    #[error("unsupported Gingr webhook entity_id representation: {observed_type}")]
    UnsupportedEntityId { observed_type: String },
    #[error("malformed Gingr webhook signature: {reason}")]
    MalformedSignature { reason: String },
    #[error("Gingr webhook signature mismatch")]
    SignatureMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ack {
    Processed,
    RejectedPermanently,
    RetryableFailure,
    RetryableStatus(response::HttpStatus),
}

impl Ack {
    pub fn retryable_status(status: response::HttpStatus) -> Self {
        if status.is_gingr_retry_override_allowed() {
            Self::RetryableStatus(status)
        } else {
            Self::RetryableFailure
        }
    }

    pub fn http_status(&self) -> response::HttpStatus {
        match self {
            Self::Processed => response::HttpStatus::OK,
            Self::RejectedPermanently => response::HttpStatus::FORBIDDEN,
            Self::RetryableFailure => response::HttpStatus::INTERNAL_SERVER_ERROR,
            Self::RetryableStatus(status) => *status,
        }
    }
}

fn verify_signature(
    key: &SignatureKey,
    webhook_type: &str,
    entity_id: &str,
    entity_type: &str,
    supplied_signature: &str,
) -> VerificationResult<()> {
    let supplied = decode_lower_hex_sha256(supplied_signature)?;
    let mut mac = HmacSha256::new_from_slice(key.expose_for_verification().as_bytes())
        .expect("HMAC accepts keys of any size");
    mac.update(webhook_type.as_bytes());
    mac.update(entity_id.as_bytes());
    mac.update(entity_type.as_bytes());
    let expected = mac.finalize().into_bytes();

    if expected.as_slice().ct_eq(&supplied).into() {
        Ok(())
    } else {
        Err(VerificationError::SignatureMismatch)
    }
}

fn decode_lower_hex_sha256(raw: &str) -> VerificationResult<[u8; 32]> {
    if raw.len() != 64 {
        return Err(VerificationError::MalformedSignature {
            reason: "expected 64 lowercase hex characters".to_owned(),
        });
    }

    let mut decoded = [0_u8; 32];
    for (index, pair) in raw.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_lower_hex_nibble(pair[0])?;
        let low = decode_lower_hex_nibble(pair[1])?;
        decoded[index] = (high << 4) | low;
    }
    Ok(decoded)
}

fn decode_lower_hex_nibble(byte: u8) -> VerificationResult<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(VerificationError::MalformedSignature {
            reason: "signature must be lowercase hex".to_owned(),
        }),
    }
}

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
