# Gingr service-domain provider surfaces

This note records which Gingr API surfaces are represented as provider DTOs/mappers for the service-domain modules. DTOs intentionally speak Gingr/API language; mappers are the boundary promotion into `domain::service::*` types.

## Retail

Documented endpoint: `GET /api/v1/get_all_retail_items` (`endpoint::commerce_retail::get::AllRetailItems`).

The SDK models this as `dto::retail::Item`, preserving unknown provider fields with `serde(flatten)`. `mapping::retail::product_candidate` promotes the provider item into a retail domain candidate containing:

- `dto::retail::ItemId` for the Gingr item identity.
- `domain::service::retail::ProductName` for the provider item name.
- `domain::service::retail::Product` for the promoted SKU/category.
- `domain::service::retail::OfferingStatus` from the provider active flag.

The mapper does not invent inventory policy from `quantity_on_hand`; Gingr's article documents the endpoint existence but not enough stock semantics to choose a domain inventory policy.

## Grooming

Documented endpoint family: `GET /api/v1/get_services_by_type` returns allowable additional services for a reservation type, but the harvested Gingr documentation does not identify a grooming-specific payload shape or a stable service taxonomy.

The SDK therefore records `dto::grooming::provider_surface()` as `ProviderSurface::NoDocumentedServiceDto { endpoint: "get_services_by_type" }` and keeps `grooming` in `endpoint::catalog::semantic_mapping_gaps()` rather than introducing fake grooming DTOs.

## Training

Documented endpoint family: `GET /api/v1/get_services_by_type` may include training-like add-on services depending on the tenant's reservation types, but the harvested Gingr documentation does not define a training enrollment/session/progress payload.

The SDK records `dto::training::provider_surface()` as `ProviderSurface::NoDocumentedServiceDto { endpoint: "get_services_by_type" }` and keeps `training` in `endpoint::catalog::semantic_mapping_gaps()` rather than inventing fake training DTOs.
