use std::collections::BTreeMap;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(transparent)]
/// Gingr retail item identifier as it appears in provider DTOs.
pub struct ItemId(u64);

impl ItemId {
    /// Creates the wrapper from an already validated value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the provider numeric identifier carried by this wrapper.
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
/// Gingr retail item DTO with known fields plus quarantined unknown provider keys.
pub struct Item {
    /// Persisted id value for this record.
    pub id: ItemId,
    #[serde(default)]
    /// Human-readable display name paired with the stable code.
    pub name: Option<String>,
    #[serde(default)]
    /// Persisted sku value for this record.
    pub sku: Option<String>,
    #[serde(default, alias = "retail_category")]
    /// Persisted category value for this record.
    pub category: Option<String>,
    #[serde(default)]
    /// Persisted active value for this record.
    pub active: Option<bool>,
    #[serde(default)]
    /// Persisted quantity on hand value for this record.
    pub quantity_on_hand: Option<u32>,
    #[serde(flatten)]
    /// Persisted unknown value for this record.
    pub unknown: BTreeMap<String, serde_json::Value>,
}
