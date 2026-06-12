use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Partner {
    VirbacCalmCare,
    PurinaProPlanVeterinarySupplements,
    PurinaEnBoardingDiet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogRelationship {
    pub partner: Partner,
    pub external_catalog_managed: bool,
}
