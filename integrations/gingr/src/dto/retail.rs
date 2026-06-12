use std::collections::BTreeMap;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct ItemId(u64);

impl ItemId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Item {
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default, alias = "retail_category")]
    pub category: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub quantity_on_hand: Option<u32>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, serde_json::Value>,
}
