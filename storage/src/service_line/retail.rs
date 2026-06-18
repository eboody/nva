use serde::{Deserialize, Serialize};

use domain::retail::product;

/// Storage shape for a migrated retail service contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::retail::Contract);

impl From<domain::retail::Contract> for ContractRecord {
    fn from(value: domain::retail::Contract) -> Self {
        Self(value)
    }
}

impl From<ContractRecord> for domain::retail::Contract {
    fn from(record: ContractRecord) -> Self {
        record.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Storage-facing retail partner product code.
pub enum PartnerCode {
    /// Stable storage code for virbac calm care.
    VirbacCalmCare,
    /// Stable storage code for purina pro plan veterinary supplements.
    PurinaProPlanVeterinarySupplements,
    /// Stable storage code for purina en boarding diet.
    PurinaEnBoardingDiet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Storage-facing retail product category code.
pub enum ProductCategoryCode {
    /// Stable storage code for supplement.
    Supplement,
    /// Stable storage code for in house diet.
    InHouseDiet,
    /// Stable storage code for personalized upsell.
    PersonalizedUpsell,
}

impl From<PartnerCode> for domain::retail::Partner {
    fn from(value: PartnerCode) -> Self {
        match value {
            PartnerCode::VirbacCalmCare => Self::VirbacCalmCare,
            PartnerCode::PurinaProPlanVeterinarySupplements => {
                Self::PurinaProPlanVeterinarySupplements
            }
            PartnerCode::PurinaEnBoardingDiet => Self::PurinaEnBoardingDiet,
        }
    }
}

impl From<domain::retail::Partner> for PartnerCode {
    fn from(value: domain::retail::Partner) -> Self {
        match value {
            domain::retail::Partner::VirbacCalmCare => Self::VirbacCalmCare,
            domain::retail::Partner::PurinaProPlanVeterinarySupplements => {
                Self::PurinaProPlanVeterinarySupplements
            }
            domain::retail::Partner::PurinaEnBoardingDiet => Self::PurinaEnBoardingDiet,
        }
    }
}

impl From<ProductCategoryCode> for product::Category {
    fn from(value: ProductCategoryCode) -> Self {
        match value {
            ProductCategoryCode::Supplement => Self::Supplement,
            ProductCategoryCode::InHouseDiet => Self::InHouseDiet,
            ProductCategoryCode::PersonalizedUpsell => Self::PersonalizedUpsell,
        }
    }
}

impl From<product::Category> for ProductCategoryCode {
    fn from(value: product::Category) -> Self {
        match value {
            product::Category::Supplement => Self::Supplement,
            product::Category::InHouseDiet => Self::InHouseDiet,
            product::Category::PersonalizedUpsell => Self::PersonalizedUpsell,
        }
    }
}
