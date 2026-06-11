use bytes::Bytes;
use std::{collections::BTreeMap, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Raw {
    status: u16,
    body: Bytes,
}

impl Raw {
    pub fn new(status: u16, body: impl Into<Bytes>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Envelope<T> {
    pub success: Option<bool>,
    pub error: Option<String>,
    pub data: T,
    #[serde(flatten)]
    pub unknown: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ProviderPayload(pub serde_json::Value);

impl fmt::Display for ProviderPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<provider payload quarantined>")
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OwnerRecord {
    pub id: u64,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
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
    pub id: u64,
    #[serde(default)]
    pub owner_id: Option<u64>,
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
    pub id: u64,
    #[serde(default)]
    pub owner_id: Option<u64>,
    #[serde(default)]
    pub animal_id: Option<u64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ReferenceRecord {
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, serde_json::Value>,
}
