use crate::dto;
use domain::retail;

use super::{Error, ProviderField, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Retail mapping candidate produced from Gingr item DTO fields.
pub struct ProductCandidate {
    /// Persisted provider item id value for this record.
    pub provider_item_id: dto::retail::ItemId,
    /// Human-readable display name paired with the stable code.
    pub name: retail::product::Name,
    /// Persisted product value for this record.
    pub product: retail::Product,
    /// Persisted status value for this record.
    pub status: retail::OfferingStatus,
}

/// Extracts retail fields Gingr exposed for product matching and merchandising.
pub fn product_candidate(item: &dto::retail::Item) -> Result<ProductCandidate> {
    let name = item
        .name
        .as_deref()
        .ok_or(Error::MissingRequiredProviderField {
            field: ProviderField::RetailItemName,
        })?;
    let name = retail::product::Name::try_new(name).map_err(|err| Error::InvalidDomainValue {
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
        .unwrap_or(retail::product::Category::PersonalizedUpsell);
    let status = if item.active.unwrap_or(true) {
        retail::OfferingStatus::Active
    } else {
        retail::OfferingStatus::Inactive
    };

    Ok(ProductCandidate {
        provider_item_id: item.id,
        name,
        product: retail::Product::new(sku, category),
        status,
    })
}

fn promote_category(value: &str) -> Result<retail::product::Category> {
    match value.trim().to_ascii_lowercase().as_str() {
        "supplement" | "supplements" => Ok(retail::product::Category::Supplement),
        "in_house_diet" | "in-house diet" | "in house diet" | "food" | "diet" => {
            Ok(retail::product::Category::InHouseDiet)
        }
        "personalized_upsell" | "personalized upsell" | "upsell" | "retail" => {
            Ok(retail::product::Category::PersonalizedUpsell)
        }
        _ => Err(Error::InvalidDomainValue {
            field: ProviderField::RetailItemCategory,
            reason: format!("unsupported Gingr retail category {value:?}"),
        }),
    }
}
