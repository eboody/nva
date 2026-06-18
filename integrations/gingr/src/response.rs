use crate::endpoint;
use bytes::Bytes;
use std::{collections::BTreeMap, fmt};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(transparent)]
/// HTTP status wrapper used by Gingr transport and webhook acknowledgements.
pub struct HttpStatus(u16);

impl HttpStatus {
    /// HTTP 200 acknowledgement used for accepted Gingr responses and webhooks.
    pub const OK: Self = Self(200);
    /// HTTP 403 status returned when Gingr rejects authorization or signature checks fail.
    pub const FORBIDDEN: Self = Self(403);
    /// HTTP 500 status used when downstream processing fails after a Gingr request.
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);

    /// Creates the wrapper from an already validated value.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the numeric HTTP status code.
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    /// Classifies statuses that should trigger Gingr retry handling.
    pub const fn is_gingr_retry_override_allowed(self) -> bool {
        matches!(self.0, 100..=599) && self.0 != Self::OK.0 && self.0 != Self::FORBIDDEN.0
    }
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Raw Gingr response body paired with status for later DTO decoding.
pub struct Raw {
    status: HttpStatus,
    body: Bytes,
}

impl Raw {
    /// Builds the validated storage wrapper for a known-good value.
    pub fn new(status: HttpStatus, body: impl Into<Bytes>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    /// Returns the HTTP status reported by Gingr.
    pub fn status(&self) -> HttpStatus {
        self.status
    }

    /// Returns the response body decoded from the Gingr transport.
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

/// Provider response envelope boundary for Gingr provider payloads.
pub mod provider {
    use std::fmt;

    #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    #[serde(transparent)]
    /// Gingr error response DTO preserving provider diagnostics.
    pub struct Error {
        detail: String,
    }

    impl Error {
        /// Builds the validated storage wrapper for a known-good value.
        pub fn new(detail: impl Into<String>) -> Self {
            Self {
                detail: detail.into(),
            }
        }

        /// Returns the provider error detail string, if Gingr supplied one.
        pub fn detail(&self) -> &str {
            &self.detail
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(&self.detail)
        }
    }

    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    /// Provider-specific webhook payload body retained for downstream DTO mapping.
    pub struct Payload(pub serde_json::Value);

    impl fmt::Display for Payload {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("<provider payload quarantined>")
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    #[serde(transparent)]
    /// Gingr owner email DTO used in response mapping.
    pub struct Email(String);

    impl Email {
        /// Builds the validated storage wrapper for a known-good value.
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }

        /// Returns the normalized provider or storage string slice.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl fmt::Display for Email {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(&self.0)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
/// Raw Gingr webhook envelope before signature verification and required-field promotion.
pub struct Envelope<T> {
    /// Persisted success value for this record.
    pub success: Option<bool>,
    /// Persisted error value for this record.
    pub error: Option<provider::Error>,
    /// Persisted data value for this record.
    pub data: T,
    #[serde(flatten)]
    /// Persisted unknown value for this record.
    pub unknown: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
/// Gingr owner response DTO before customer-domain mapping.
pub struct OwnerRecord {
    /// Persisted id value for this record.
    pub id: endpoint::OwnerId,
    #[serde(default)]
    /// Persisted first name value for this record.
    pub first_name: Option<String>,
    #[serde(default)]
    /// Persisted last name value for this record.
    pub last_name: Option<String>,
    #[serde(default)]
    /// Persisted email value for this record.
    pub email: Option<provider::Email>,
    #[serde(default, alias = "cell")]
    /// Persisted cell phone value for this record.
    pub cell_phone: Option<String>,
    #[serde(flatten)]
    /// Persisted unknown value for this record.
    pub unknown: BTreeMap<String, serde_json::Value>,
}

impl OwnerRecord {
    /// Builds a readable owner name from first and last name fields.
    pub fn display_name(&self) -> Option<String> {
        let joined = [self.first_name.as_deref(), self.last_name.as_deref()]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if joined.is_empty() {
            None
        } else {
            Some(joined)
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
/// Gingr animal response DTO before pet-domain mapping.
pub struct AnimalRecord {
    /// Persisted id value for this record.
    pub id: endpoint::AnimalId,
    #[serde(default)]
    /// Persisted owner id value for this record.
    pub owner_id: Option<endpoint::OwnerId>,
    #[serde(default)]
    /// Human-readable display name paired with the stable code.
    pub name: Option<String>,
    #[serde(default)]
    /// Persisted species value for this record.
    pub species: Option<String>,
    #[serde(default)]
    /// Persisted birthday value for this record.
    pub birthday: Option<String>,
    #[serde(flatten)]
    /// Persisted unknown value for this record.
    pub unknown: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
/// Gingr reservation response DTO used to reconcile bookings and source evidence.
pub struct ReservationRecord {
    /// Persisted id value for this record.
    pub id: endpoint::ReservationId,
    #[serde(default)]
    /// Persisted owner id value for this record.
    pub owner_id: Option<endpoint::OwnerId>,
    #[serde(default)]
    /// Persisted animal id value for this record.
    pub animal_id: Option<endpoint::AnimalId>,
    #[serde(default)]
    /// Persisted status value for this record.
    pub status: Option<String>,
    #[serde(flatten)]
    /// Persisted unknown value for this record.
    pub unknown: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
/// Gingr reference-data DTO for lookup tables such as locations or vets.
pub struct ReferenceRecord {
    /// Persisted id value for this record.
    pub id: endpoint::ReferenceId,
    #[serde(default)]
    /// Human-readable display name paired with the stable code.
    pub name: Option<String>,
    #[serde(flatten)]
    /// Persisted unknown value for this record.
    pub unknown: BTreeMap<String, serde_json::Value>,
}
