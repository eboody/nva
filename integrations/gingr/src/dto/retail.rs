use std::collections::BTreeMap;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(transparent)]
/// Gingr retail item identifier as it appears in provider DTOs.
pub struct ItemId(u64);

impl ItemId {
    /// Wraps an already-observed Gingr identifier without claiming anything beyond provider provenance.
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
    /// Provider record identifier observed in the Gingr payload.
    pub id: ItemId,
    #[serde(default)]
    /// Provider display label retained for operator context; NVA-specific naming rules are applied downstream.
    pub name: Option<String>,
    #[serde(default)]
    /// Provider SKU/code observed for the retail item and used as retail source evidence.
    pub sku: Option<String>,
    #[serde(default, alias = "retail_category")]
    /// Provider category label observed for the retail item before NVA retail taxonomy validation.
    pub category: Option<String>,
    #[serde(default)]
    /// Provider active flag observed for the retail item, not an NVA-approved sellability decision.
    pub active: Option<bool>,
    #[serde(default)]
    /// Provider quantity-on-hand observation for inventory workflows; reconciliation remains downstream.
    pub quantity_on_hand: Option<u32>,
    #[serde(flatten)]
    /// Extra provider fields preserved for audit and future mapping without becoming validated NVA facts.
    pub unknown: BTreeMap<String, serde_json::Value>,
}
