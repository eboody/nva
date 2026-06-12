use serde::{Deserialize, Serialize};

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
pub enum PartnerCode {
    VirbacCalmCare,
    PurinaProPlanVeterinarySupplements,
    PurinaEnBoardingDiet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductCategoryCode {
    Supplement,
    InHouseDiet,
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

impl From<ProductCategoryCode> for domain::retail::ProductCategory {
    fn from(value: ProductCategoryCode) -> Self {
        match value {
            ProductCategoryCode::Supplement => Self::Supplement,
            ProductCategoryCode::InHouseDiet => Self::InHouseDiet,
            ProductCategoryCode::PersonalizedUpsell => Self::PersonalizedUpsell,
        }
    }
}

impl From<domain::retail::ProductCategory> for ProductCategoryCode {
    fn from(value: domain::retail::ProductCategory) -> Self {
        match value {
            domain::retail::ProductCategory::Supplement => Self::Supplement,
            domain::retail::ProductCategory::InHouseDiet => Self::InHouseDiet,
            domain::retail::ProductCategory::PersonalizedUpsell => Self::PersonalizedUpsell,
        }
    }
}
