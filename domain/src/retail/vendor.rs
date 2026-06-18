//! Vendor contracts for partner products and external catalog-management flags.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Retail partner product line used for recommendation and catalog relationship documentation.
pub enum Partner {
    /// Virbac calm care retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    VirbacCalmCare,
    /// Purina pro plan veterinary supplements retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    PurinaProPlanVeterinarySupplements,
    /// Purina en boarding diet retail operational signal for inventory, POS, reorder, recommendation, or review handling.
    PurinaEnBoardingDiet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Relationship between a partner product line and whether an external vendor manages the catalog facts.
pub struct CatalogRelationship {
    /// Source-derived partner carried by this retail contract.
    pub partner: Partner,
    /// Source-derived external catalog managed carried by this retail contract.
    pub external_catalog_managed: bool,
}
