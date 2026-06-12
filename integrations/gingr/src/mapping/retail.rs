use crate::dto;
use domain::service::retail;

use super::{Error, ProviderField, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductCandidate {
    pub provider_item_id: dto::retail::ItemId,
    pub name: retail::ProductName,
    pub product: retail::Product,
    pub status: retail::OfferingStatus,
}

pub fn product_candidate(item: &dto::retail::Item) -> Result<ProductCandidate> {
    let name = item
        .name
        .as_deref()
        .ok_or(Error::MissingRequiredProviderField {
            field: ProviderField::RetailItemName,
        })?;
    let name = retail::ProductName::try_new(name).map_err(|err| Error::InvalidDomainValue {
        field: ProviderField::RetailItemName,
        reason: err.to_string(),
    })?;

    let sku = item
        .sku
        .as_deref()
        .ok_or(Error::MissingRequiredProviderField {
            field: ProviderField::RetailItemSku,
        })?;
    let sku = retail::Sku::try_new(sku).map_err(|err| Error::InvalidDomainValue {
        field: ProviderField::RetailItemSku,
        reason: err.to_string(),
    })?;

    let category = item
        .category
        .as_deref()
        .map(promote_category)
        .transpose()?
        .unwrap_or(retail::ProductCategory::PersonalizedUpsell);
    let status = if item.active.unwrap_or(true) {
        retail::OfferingStatus::Active
    } else {
        retail::OfferingStatus::Inactive
    };

    Ok(ProductCandidate {
        provider_item_id: dto::retail::ItemId::new(item.id),
        name,
        product: retail::Product::new(sku, category),
        status,
    })
}

fn promote_category(value: &str) -> Result<retail::ProductCategory> {
    match value.trim().to_ascii_lowercase().as_str() {
        "supplement" | "supplements" => Ok(retail::ProductCategory::Supplement),
        "in_house_diet" | "in-house diet" | "in house diet" | "food" | "diet" => {
            Ok(retail::ProductCategory::InHouseDiet)
        }
        "personalized_upsell" | "personalized upsell" | "upsell" | "retail" => {
            Ok(retail::ProductCategory::PersonalizedUpsell)
        }
        _ => Err(Error::InvalidDomainValue {
            field: ProviderField::RetailItemCategory,
            reason: format!("unsupported Gingr retail category {value:?}"),
        }),
    }
}
