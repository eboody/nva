#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(transparent)]
/// Gingr retail item identifier as it appears in provider DTOs.
pub struct ItemId(u64);

impl ItemId {}
