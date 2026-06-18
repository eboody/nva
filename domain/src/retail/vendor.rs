use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Domain vocabulary for partner decisions in retail workflows.
pub enum Partner {
    /// Virbac calm care retail inventory, POS, reorder, or recommendation signal.
    VirbacCalmCare,
    /// Purina pro plan veterinary supplements retail inventory, POS, reorder, or recommendation signal.
    PurinaProPlanVeterinarySupplements,
    /// Purina en boarding diet retail inventory, POS, reorder, or recommendation signal.
    PurinaEnBoardingDiet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Typed catalog relationship domain value that keeps raw primitives out of retail workflows.
pub struct CatalogRelationship {
    /// Partner fact promoted into this retail contract.
    pub partner: Partner,
    /// External catalog managed fact promoted into this retail contract.
    pub external_catalog_managed: bool,
}
