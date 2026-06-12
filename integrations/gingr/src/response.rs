use crate::endpoint;
use bytes::Bytes;
use std::{collections::BTreeMap, fmt};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(transparent)]
pub struct HttpStatus(u16);

impl HttpStatus {
    pub const OK: Self = Self(200);
    pub const FORBIDDEN: Self = Self(403);
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }

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
pub struct Raw {
    status: HttpStatus,
    body: Bytes,
}

impl Raw {
    pub fn new(status: HttpStatus, body: impl Into<Bytes>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    pub fn status(&self) -> HttpStatus {
        self.status
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

pub mod provider {
    use std::fmt;

    #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    #[serde(transparent)]
    pub struct Error {
        detail: String,
    }

    impl Error {
        pub fn new(detail: impl Into<String>) -> Self {
            Self {
                detail: detail.into(),
            }
        }

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
    pub struct Payload(pub serde_json::Value);

    impl fmt::Display for Payload {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("<provider payload quarantined>")
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    #[serde(transparent)]
    pub struct Email(String);

    impl Email {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }

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
pub struct Envelope<T> {
    pub success: Option<bool>,
    pub error: Option<provider::Error>,
    pub data: T,
    #[serde(flatten)]
    pub unknown: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OwnerRecord {
    pub id: endpoint::OwnerId,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub email: Option<provider::Email>,
    #[serde(default, alias = "cell")]
    pub cell_phone: Option<String>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, serde_json::Value>,
}

impl OwnerRecord {
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
pub struct AnimalRecord {
    pub id: endpoint::AnimalId,
    #[serde(default)]
    pub owner_id: Option<endpoint::OwnerId>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub species: Option<String>,
    #[serde(default)]
    pub birthday: Option<String>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ReservationRecord {
    pub id: endpoint::ReservationId,
    #[serde(default)]
    pub owner_id: Option<endpoint::OwnerId>,
    #[serde(default)]
    pub animal_id: Option<endpoint::AnimalId>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ReferenceRecord {
    pub id: endpoint::ReferenceId,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, serde_json::Value>,
}
