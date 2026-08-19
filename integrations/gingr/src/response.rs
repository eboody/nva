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
    /// Captures the raw Gingr status and body so decoding, retry decisions, and audit can happen later.
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

/// Provider response envelopes that preserve Gingr success/error/data fields before DTO decoding.
pub mod provider {
    use std::fmt;

    #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    #[serde(transparent)]
    /// Gingr error response DTO preserving provider diagnostics.
    pub struct Error {
        detail: String,
    }

    impl Error {
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

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
/// Gingr owner response DTO before customer-domain mapping.
pub struct OwnerRecord {
    /// Provider record identifier observed in the Gingr payload.
    pub id: endpoint::OwnerId,
    #[serde(default)]
    /// Owner first name observed in Gingr and used only as provider contact context.
    pub first_name: Option<String>,
    #[serde(default)]
    /// Owner last name observed in Gingr and used only as provider contact context.
    pub last_name: Option<String>,
    #[serde(default)]
    /// Email address observed from Gingr and carried as customer-contact evidence.
    pub email: Option<provider::Email>,
    /// Owner cell-phone value observed in Gingr and carried as contact evidence.
    pub cell_phone: Option<String>,
    #[serde(flatten)]
    /// Extra provider fields preserved for audit and future mapping without becoming validated NVA facts.
    pub unknown: BTreeMap<String, serde_json::Value>,
}

impl OwnerRecord {}
