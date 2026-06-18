# `gingr::dto`

`gingr::dto` is the Gingr provider-adapter module for provider payload shapes that are documented or fixture-backed enough to deserialize directly. It owns provider-shaped DTO records, provider DTO id wrappers, unknown-field retention for DTO payload drift, and explicit provider-surface gap markers for endpoint families where the source does not yet justify a Rust DTO.

This module is not the domain model. A DTO such as [`gingr::dto::retail::Item`](./retail.rs) preserves a Gingr retail item payload so the adapter can review and map source data safely; domain truth lives in `domain::*`, mapping decisions live in [`gingr::mapping`](../mapping/mod.rs), application workflows live in [`app`](../../../../app/README.md), and durable projections live in [`storage`](../../../../storage/README.md).

## Module navigation

Start at [`mod.rs`](./mod.rs). It declares the DTO family modules and the shared [`gingr::dto::ProviderSurface`](./mod.rs) marker used when the documented provider surface exists but the crate does not have a supported DTO.

DTO family files:

- [`retail.rs`](./retail.rs) defines [`gingr::dto::retail::ItemId`](./retail.rs) and [`gingr::dto::retail::Item`](./retail.rs). `Item` is provider-shaped retail catalog data: provider item id, optional name, optional SKU, optional category/`retail_category`, optional active flag, optional quantity-on-hand, and a flattened `unknown` map for fields this crate does not semantically understand yet.
- [`grooming.rs`](./grooming.rs) returns `ProviderSurface::NoDocumentedServiceDto { endpoint: "get_services_by_type" }`. That is a source-provenance statement: the endpoint is known, but this crate does not claim a documented Gingr grooming service DTO.
- [`training.rs`](./training.rs) returns the same `ProviderSurface::NoDocumentedServiceDto` marker for `get_services_by_type`. Do not add a training DTO until provider documentation or fixtures identify a stable payload shape.

Related provider response records that are not under `dto` live in [`../response.rs`](../response.rs). For example, [`gingr::response::OwnerRecord`](../response.rs), [`AnimalRecord`](../response.rs), [`ReservationRecord`](../response.rs), and [`ReferenceRecord`](../response.rs) are general response records used before mapping.

## Provider payload shapes and sensitivity boundaries

`gingr::dto` preserves provider facts at the adapter boundary:

1. Provider ids such as [`gingr::dto::retail::ItemId`](./retail.rs) are Gingr-scoped identifiers. They are not canonical `domain::entities::*` ids and should not be persisted as domain identity without explicit provenance or a storage/source-reference design.
2. Optional provider fields stay optional until a mapping function proves a domain value. For retail, [`gingr::mapping::retail::product_candidate`](../mapping/retail.rs) requires a retail item name and SKU before constructing `domain::retail` values.
3. Unknown fields are retained on [`gingr::dto::retail::Item::unknown`](./retail.rs). Unknown fields are evidence for provider drift or future mapping work, not a license to read business semantics from arbitrary JSON.
4. The current retail DTO fields are catalog/inventory-shaped (`name`, `sku`, `category`, `active`, `quantity_on_hand`). They are not marked as payment-sensitive in source. Payment-sensitive response classification currently appears on transaction endpoint requests in [`../endpoint/commerce_retail.rs`](../endpoint/commerce_retail.rs), not in this DTO module.
5. Grooming and training intentionally expose gap markers instead of placeholder DTOs. That keeps review safer: maintainers can see that service DTO semantics are unsupported instead of relying on invented fields.

If a new DTO may contain phone numbers, email addresses, payment data, staff notes, customer notes, medication/health facts, or custom-field values, document that sensitivity here and make sure transport/response logging redacts or quarantines it before review.

## Source provenance

The DTO module's provenance is provider-source provenance, not business truth:

- Endpoint request coverage lives in [`../endpoint`](../endpoint/mod.rs), including the provider catalog inventory in [`../endpoint/catalog.rs`](../endpoint/catalog.rs).
- Provider response envelopes and general records live in [`../response.rs`](../response.rs), where unknown top-level and record fields are retained.
- Provider documentation and fixture inventory live in [`../../../../docs/integrations/gingr/README.md`](../../../../docs/integrations/gingr/README.md) and webhook fixture docs live in [`../../../../docs/integrations/gingr/fixtures/webhooks/README.md`](../../../../docs/integrations/gingr/fixtures/webhooks/README.md).
- Semantic promotion happens only after DTO/response data crosses into [`../mapping`](../mapping/mod.rs). When no mapping exists, the DTO remains provider evidence only.

## Type/module map

| Concept | Public type/module path | Defined in | Role |
| --- | --- | --- | --- |
| DTO module registry | `gingr::dto` | [`mod.rs`](./mod.rs) | Declares provider DTO modules and provider-surface markers. |
| Provider surface gap marker | `gingr::dto::ProviderSurface` | [`mod.rs`](./mod.rs) | Records known endpoints without a documented/supported DTO shape. |
| Retail DTO module | `gingr::dto::retail` | [`retail.rs`](./retail.rs) | Owns the provider-shaped retail item payload. |
| Retail provider item id | `gingr::dto::retail::ItemId` | [`retail.rs`](./retail.rs) | Gingr-scoped numeric item id wrapper. |
| Retail provider item record | `gingr::dto::retail::Item` | [`retail.rs`](./retail.rs) | Deserializes retail catalog fields and retains unknown provider fields. |
| Grooming DTO gap | `gingr::dto::grooming::provider_surface` | [`grooming.rs`](./grooming.rs) | States that `get_services_by_type` does not have a documented Gingr grooming service DTO here. |
| Training DTO gap | `gingr::dto::training::provider_surface` | [`training.rs`](./training.rs) | States that `get_services_by_type` does not have a documented Gingr training service DTO here. |
| Raw/general provider records | `gingr::response::{OwnerRecord, AnimalRecord, ReservationRecord, ReferenceRecord}` | [`../response.rs`](../response.rs) | Provider response records outside the DTO submodule; mapped by `gingr::mapping` when supported. |
| Retail semantic promotion | `gingr::mapping::retail::{ProductCandidate, product_candidate}` | [`../mapping/retail.rs`](../mapping/retail.rs) | Promotes retail DTO fields into `domain::retail` values after validation. |

## Labor-cost and automation role

`gingr::dto` reduces maintainer and manager labor by making provider payload review explicit before automation consumes source data:

1. Retail item DTOs normalize the documented catalog payload shape into a typed Rust record, so downstream mapping can validate name/SKU/category once instead of asking staff to inspect raw catalog JSON repeatedly.
2. Unknown-field retention gives reviewers evidence when Gingr adds or changes payload fields; the adapter can surface drift without silently corrupting domain projections.
3. Grooming/training gap markers avoid false automation confidence. A maintainer sees that no service DTO exists yet and can route the work to provider-doc/fixture collection before adding mappings.
4. The DTO/mapping split supports safer exception triage: clean, required provider fields can move forward through typed mappings, while missing or unsupported provider fields become explicit mapping errors or documented gaps.

## Cross-crate relationships

- [`integrations/gingr`](../../README.md) is the crate-level guide for config, endpoint, transport, response, webhook, DTO, and mapping boundaries.
- [`gingr::endpoint`](../endpoint/README.md) builds provider requests; DTOs only describe response payload shapes after a provider call or fixture supplies data.
- [`gingr::response`](../response.rs) owns raw response wrappers and general provider records. `dto` should not duplicate those shapes unless a provider endpoint has a distinct documented DTO.
- [`gingr::mapping`](../mapping/mod.rs) promotes supported provider DTO/response fields into domain candidates and typed errors.
- [`domain`](../../../../domain/README.md) owns semantic business concepts. Retail target types live in [`domain/src/retail/mod.rs`](../../../../domain/src/retail/mod.rs) and [`domain/src/retail/product.rs`](../../../../domain/src/retail/product.rs); customer and pet target values used by response mappings live in [`domain/src/customer.rs`](../../../../domain/src/customer.rs) and [`domain/src/pet.rs`](../../../../domain/src/pet.rs).
- [`storage`](../../../../storage/README.md) owns durable records and source evidence. Service-line storage code mappings are documented in [`storage/src/service_line/README.md`](../../../../storage/src/service_line/README.md); do not persist provider DTOs directly just because they deserialize.
- [`app`](../../../../app/README.md) composes normalized facts into draft/review workflows. App modules should consume domain or storage-normalized values, not raw `gingr::dto` records.
- [`docs/integrations/gingr`](../../../../docs/integrations/gingr/README.md) is the provider documentation/fixture inventory to consult before adding a new DTO.

## Maintainer notes

- Preserve provider boundary names in prose. Prefer `gingr::dto::retail::Item` and `gingr::dto::retail::ItemId` when discussing provider records; prefer `domain::retail::Product` only after mapping has validated source fields.
- Add DTOs only when the provider payload shape is documented or fixture-backed enough to deserialize safely. Otherwise extend `ProviderSurface` or the catalog gap documentation instead of inventing fields.
- Keep sensitivity boundaries current. If a DTO grows customer, payment, care, medical, staff, or custom-field data, update this README and any redaction/logging path that could expose it.
