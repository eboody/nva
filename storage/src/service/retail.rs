use serde::{Deserialize, Serialize};

/// Storage shape for a migrated retail service contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractRecord(pub domain::service::retail::Contract);

impl From<domain::service::retail::Contract> for ContractRecord {
    fn from(value: domain::service::retail::Contract) -> Self {
        Self(value)
    }
}

impl From<ContractRecord> for domain::service::retail::Contract {
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

impl From<PartnerCode> for domain::service::retail::Partner {
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

impl From<domain::service::retail::Partner> for PartnerCode {
    fn from(value: domain::service::retail::Partner) -> Self {
        match value {
            domain::service::retail::Partner::VirbacCalmCare => Self::VirbacCalmCare,
            domain::service::retail::Partner::PurinaProPlanVeterinarySupplements => {
                Self::PurinaProPlanVeterinarySupplements
            }
            domain::service::retail::Partner::PurinaEnBoardingDiet => Self::PurinaEnBoardingDiet,
        }
    }
}

impl From<ProductCategoryCode> for domain::service::retail::ProductCategory {
    fn from(value: ProductCategoryCode) -> Self {
        match value {
            ProductCategoryCode::Supplement => Self::Supplement,
            ProductCategoryCode::InHouseDiet => Self::InHouseDiet,
            ProductCategoryCode::PersonalizedUpsell => Self::PersonalizedUpsell,
        }
    }
}

impl From<domain::service::retail::ProductCategory> for ProductCategoryCode {
    fn from(value: domain::service::retail::ProductCategory) -> Self {
        match value {
            domain::service::retail::ProductCategory::Supplement => Self::Supplement,
            domain::service::retail::ProductCategory::InHouseDiet => Self::InHouseDiet,
            domain::service::retail::ProductCategory::PersonalizedUpsell => {
                Self::PersonalizedUpsell
            }
        }
    }
}
