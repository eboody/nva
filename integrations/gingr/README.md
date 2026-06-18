# `integrations/gingr`

`integrations/gingr` is the Gingr provider adapter crate. It owns the provider-facing boundary for Gingr configuration, endpoint request construction, transport seams, response envelopes, webhook verification, provider DTOs, and the first mapping step from provider records into semantic `domain::*` candidates.

Start at [`src/lib.rs`](./src/lib.rs). The crate root exposes [`config`](./src/config.rs), [`endpoint`](./src/endpoint/mod.rs), [`transport`](./src/transport.rs), [`response`](./src/response.rs), [`webhook`](./src/webhook.rs), [`dto`](./src/dto/mod.rs), and [`mapping`](./src/mapping/mod.rs). New maintainers should read those modules in boundary order: configure a Gingr account, build endpoint requests, send/capture transport requests, parse raw provider response shapes, verify webhooks, deserialize documented provider DTOs, then explicitly promote usable provider fields into domain values.

This crate is not domain truth. Gingr ids, status strings, endpoint shapes, undocumented fields, webhooks, and API quirks are provider facts. `domain` owns normalized business concepts such as customer names, pet names, retail products, source provenance, data-quality issues, workflow decisions, and service-line policy. `storage` owns durable projection records and stable persisted codes. Treat every provider payload here as evidence that must be validated, mapped, or quarantined before downstream automation depends on it.

## Module navigation

### Crate surface and configuration

- [`src/lib.rs`](./src/lib.rs) declares the public modules and re-exports the common config types `gingr::ApiKey`, `gingr::BaseUrl`, `gingr::Provider`, and `gingr::Subdomain`.
- [`src/config.rs`](./src/config.rs) owns account-level configuration: `gingr::config::Subdomain`, `BaseUrl`, `ApiKey`, `Provider`, and `Client`. `BaseUrl::parse` only accepts `https://*.gingrapp.com/` without path/query/fragment, `Subdomain::parse` validates DNS-label-like lowercase subdomains, and `ApiKey`/`Client` redact secrets in debug/display output.

### Endpoint request builders

- [`src/endpoint/mod.rs`](./src/endpoint/mod.rs) owns shared request vocabulary: `gingr::endpoint::Request`, `Method`, `Path`, `Date`, `IsoDate`, `DateRange`, `Limit`, id wrappers such as `AnimalId`, `OwnerId`, `ReservationId`, `LocationId`, and endpoint validation errors.
- [`src/endpoint/reservations.rs`](./src/endpoint/reservations.rs) builds reservation-related request shapes, including `gingr::endpoint::Reservations`, `gingr::endpoint::reservations::reservation::SearchFilters`, `RestrictTo`, `BackOfHouse`, and `GetServicesByType`.
- [`src/endpoint/owners_animals.rs`](./src/endpoint/owners_animals.rs) builds owner/animal request shapes, including list and lookup requests (`Owners`, `Animals`, `Owner`), `SensitiveLookup`, `OwnerLookup`, form requests, custom-field lookups, and `AnimalCareInfo`. Sensitive owner lookups mark `phone`/`email` for redaction.
- [`src/endpoint/commerce_retail.rs`](./src/endpoint/commerce_retail.rs) builds commerce and retail request shapes, including `get::AllRetailItems`, subscription/package/bill-day requests, retail transaction lookup, and sensitivity classification for transaction responses.
- [`src/endpoint/reference_data.rs`](./src/endpoint/reference_data.rs) builds reference-data requests such as locations, vets, immunization types, and animal immunizations.
- [`src/endpoint/catalog.rs`](./src/endpoint/catalog.rs) records documented catalog-surface gaps and service catalog request concepts. Keep provider documentation gaps here rather than pretending the crate has a complete service DTO.
- [`src/endpoint/report_cards_files.rs`](./src/endpoint/report_cards_files.rs) builds report-card file requests.
- [`src/endpoint/labor_ops.rs`](./src/endpoint/labor_ops.rs) builds `TimeclockReport` requests for labor/operations evidence.

### Transport and raw response handling

- [`src/transport.rs`](./src/transport.rs) converts endpoint requests into transportable `RequestParts`, injects the API key with `RequestParts::with_api_key`, redacts sensitive query/form parameters through `RedactedRequest`, and defines the `Transport` trait. The default `HttpTransport` currently returns `TransportError::HttpNotImplemented`; `MockTransport` supports contract tests without live Gingr calls.
- [`src/response.rs`](./src/response.rs) owns raw HTTP response wrappers and provider response records. `gingr::response::Raw` carries `HttpStatus` plus bytes, `Envelope<T>` preserves success/error/data plus unknown provider fields, and `OwnerRecord`, `AnimalRecord`, `ReservationRecord`, and `ReferenceRecord` deserialize provider-shaped records while retaining flattened unknown fields.

### Webhooks

- [`src/webhook.rs`](./src/webhook.rs) parses and verifies webhook envelopes. It owns `SignatureKey`, `Envelope`, `Verified`, `EventType`, `EntityType`, `EntityId`, `Payload`, `Ack`, `ParseError`, and `VerificationError`. Verification normalizes string/numeric entity ids, requires `webhook_type`, `entity_id`, `entity_type`, and `signature`, and checks the HMAC-SHA256 signature with constant-time comparison before exposing `Verified` data.
- Webhook fixture documentation lives in [`../../docs/integrations/gingr/fixtures/webhooks/README.md`](../../docs/integrations/gingr/fixtures/webhooks/README.md); provider support-document inventory lives in [`../../docs/integrations/gingr/README.md`](../../docs/integrations/gingr/README.md).

### DTOs and mappings

- [`src/dto/mod.rs`](./src/dto/mod.rs) collects provider DTO modules and records documented gaps with `gingr::dto::ProviderSurface::NoDocumentedServiceDto`.
- [`src/dto/retail.rs`](./src/dto/retail.rs) owns the documented retail item shape `gingr::dto::retail::Item` and provider id wrapper `ItemId`; unknown provider fields are retained on `Item::unknown`.
- [`src/dto/grooming.rs`](./src/dto/grooming.rs) and [`src/dto/training.rs`](./src/dto/training.rs) currently return provider-surface gap markers for `get_services_by_type`; they do not define service DTOs that are not present in source.
- [`src/mapping/mod.rs`](./src/mapping/mod.rs) owns mapping errors and `ProviderField` classifications for provider fields that must be present or valid before promotion.
- [`src/mapping/customer.rs`](./src/mapping/customer.rs) promotes `response::OwnerRecord` into `mapping::customer::ContactCandidate` with `domain::customer::Name`, optional `domain::customer::Email`/`Phone`, and a derived `domain::entities::ContactChannel`.
- [`src/mapping/pet.rs`](./src/mapping/pet.rs) promotes `response::AnimalRecord` into `mapping::pet::NameCandidate` with `domain::pet::Name`.
- [`src/mapping/retail.rs`](./src/mapping/retail.rs) promotes `dto::retail::Item` into `mapping::retail::ProductCandidate` with `domain::retail::product::Name`, `domain::retail::Product`, and `domain::retail::OfferingStatus`.

## Provider facts vs domain truth

Keep this boundary explicit:

1. Provider request and response shapes belong under [`src/endpoint`](./src/endpoint/mod.rs), [`src/response.rs`](./src/response.rs), [`src/dto`](./src/dto/mod.rs), and [`src/webhook.rs`](./src/webhook.rs). These modules may preserve raw ids, endpoint names, status strings, unknown fields, and provider gaps.
2. Business semantics belong in [`../../domain`](../../domain/README.md). For example, `mapping::customer::contact_candidate` validates a Gingr owner name into `domain::customer::Name` instead of letting an arbitrary provider string become a customer profile.
3. Durable projections belong in [`../../storage`](../../storage/README.md). A Gingr DTO should not be persisted directly just because it exists; add a storage record only when the project needs a normalized durable projection or reviewable automation evidence.
4. Application workflows in [`../../app/src`](../../app/src/lib.rs) consume normalized domain/source facts. They should not parse Gingr query parameters, raw webhooks, or provider DTOs directly when a provider adapter or mapping function can do that boundary work.
5. Missing or unsupported provider fields should become typed mapping errors, source assumptions, data-quality issues, or documented provider-surface gaps. Do not silently default provider data into operationally meaningful domain values unless the mapping function says so and tests cover it.

## Type/module map

| Concept | Public type/module path | Defined in | Role |
| --- | --- | --- | --- |
| Crate module registry | `gingr` crate root | [`src/lib.rs`](./src/lib.rs) | Exposes config, endpoint, transport, response, webhook, dto, and mapping modules. |
| Account subdomain | `gingr::config::Subdomain` | [`src/config.rs`](./src/config.rs) | Validates the Gingr subdomain segment used to build a base URL. |
| API base URL | `gingr::config::BaseUrl` | [`src/config.rs`](./src/config.rs) | Restricts client base URLs to HTTPS Gingr app subdomains without path/query/fragment. |
| API key | `gingr::config::ApiKey` | [`src/config.rs`](./src/config.rs) | Secret wrapper exposed only to transport injection and redacted for formatting. |
| Provider label | `gingr::config::Provider` | [`src/config.rs`](./src/config.rs) | Identifies the provider instance for logs/config display. |
| Config client | `gingr::config::Client` | [`src/config.rs`](./src/config.rs) | Bundles base URL, API key, and provider label. |
| Endpoint request trait | `gingr::endpoint::Request` | [`src/endpoint/mod.rs`](./src/endpoint/mod.rs) | Converts typed endpoint requests into method/path/parameter parts. |
| Endpoint primitives | `gingr::endpoint::{Date, IsoDate, DateRange, Limit, Method, Path}` | [`src/endpoint/mod.rs`](./src/endpoint/mod.rs) | Shared validated request scalars. |
| Provider id wrappers | `gingr::endpoint::{OwnerId, AnimalId, ReservationId, LocationId, SpeciesId, FormId, ReferenceId}` | [`src/endpoint/mod.rs`](./src/endpoint/mod.rs) | Provider-side numeric identifiers; not canonical domain ids. |
| Reservation requests | `gingr::endpoint::Reservations`, `gingr::endpoint::reservations::reservation::{Types, WidgetData, SearchFilters}` | [`src/endpoint/reservations.rs`](./src/endpoint/reservations.rs) | Builds reservation and reservation-type API requests. |
| Owner/animal requests | `gingr::endpoint::owners_animals::{Owners, Animals, Owner, OwnerLookup, SensitiveLookup, Form, AnimalCareInfo}` | [`src/endpoint/owners_animals.rs`](./src/endpoint/owners_animals.rs) | Builds owner, animal, form, custom field, and care-info requests. |
| Retail/commerce requests | `gingr::endpoint::commerce_retail::{get, list, Transaction, ResponseSensitivity}` | [`src/endpoint/commerce_retail.rs`](./src/endpoint/commerce_retail.rs) | Builds retail, subscription, package, transaction, and commerce requests. |
| Reference-data requests | `gingr::endpoint::reference_data::{GetLocations, GetVets, GetImmunizationTypes, GetAnimalImmunizations}` | [`src/endpoint/reference_data.rs`](./src/endpoint/reference_data.rs) | Builds location/vet/immunization reference requests. |
| Labor ops request | `gingr::endpoint::labor_ops::TimeclockReport` | [`src/endpoint/labor_ops.rs`](./src/endpoint/labor_ops.rs) | Builds timeclock/labor evidence requests. |
| Request parts | `gingr::transport::RequestParts` | [`src/transport.rs`](./src/transport.rs) | Transport-ready method/path/parameters with API-key injection and redaction metadata. |
| Transport seam | `gingr::transport::Transport`, `Client<T>`, `HttpTransport`, `MockTransport` | [`src/transport.rs`](./src/transport.rs) | Separates request construction from actual HTTP execution; live HTTP is not implemented in this slice. |
| Raw response | `gingr::response::Raw`, `gingr::response::HttpStatus` | [`src/response.rs`](./src/response.rs) | Holds status/body and retry-override status classification. |
| Provider envelope | `gingr::response::Envelope<T>` | [`src/response.rs`](./src/response.rs) | Deserializes success/error/data while preserving unknown top-level fields. |
| Provider records | `gingr::response::{OwnerRecord, AnimalRecord, ReservationRecord, ReferenceRecord}` | [`src/response.rs`](./src/response.rs) | Provider-shaped records used before mapping to domain candidates. |
| Webhook verification | `gingr::webhook::{Envelope, Verified, SignatureKey, VerificationError}` | [`src/webhook.rs`](./src/webhook.rs) | Parses, validates, signs, and exposes verified webhook facts. |
| Webhook classifications | `gingr::webhook::{EventType, EntityType, EntityId, Ack}` | [`src/webhook.rs`](./src/webhook.rs) | Provider event/entity vocabulary plus acknowledgment codes. |
| DTO surface marker | `gingr::dto::ProviderSurface` | [`src/dto/mod.rs`](./src/dto/mod.rs) | Documents endpoints without a supported provider DTO in this crate. |
| Retail DTO | `gingr::dto::retail::{Item, ItemId}` | [`src/dto/retail.rs`](./src/dto/retail.rs) | Provider-shaped retail item record with unknown field retention. |
| Mapping errors | `gingr::mapping::{Error, ProviderField, Result}` | [`src/mapping/mod.rs`](./src/mapping/mod.rs) | Names provider fields and domain-promotion failures. |
| Customer mapping | `gingr::mapping::customer::{ContactCandidate, contact_candidate}` | [`src/mapping/customer.rs`](./src/mapping/customer.rs) | Promotes owner records into domain customer/contact candidate values. |
| Pet mapping | `gingr::mapping::pet::{NameCandidate, name_candidate}` | [`src/mapping/pet.rs`](./src/mapping/pet.rs) | Promotes animal records into domain pet-name candidate values. |
| Retail mapping | `gingr::mapping::retail::{ProductCandidate, product_candidate}` | [`src/mapping/retail.rs`](./src/mapping/retail.rs) | Promotes retail DTOs into domain retail product candidates. |

## Labor-cost and automation role

`integrations/gingr` reduces labor cost by making provider access reviewable and normalizable instead of requiring managers or maintainers to inspect raw Gingr payloads by hand:

1. Typed endpoint builders in [`src/endpoint`](./src/endpoint/mod.rs) prevent scattered stringly typed request construction, reduce copy/paste errors, and make parameter validation local to provider adapter code.
2. Secret redaction in [`src/config.rs`](./src/config.rs) and [`src/transport.rs`](./src/transport.rs) allows maintainers to review captured request shapes without exposing API keys, phone lookups, or email lookups.
3. Response and DTO modules preserve unknown provider fields, so source-data changes can be inspected without immediately corrupting domain projections.
4. Mapping modules turn missing provider fields or invalid domain values into typed errors (`gingr::mapping::Error`) instead of silent defaults. That supports safer exception triage: automation can route clean mapped candidates forward and reserve ambiguous records for review.
5. Webhook verification in [`src/webhook.rs`](./src/webhook.rs) keeps event-driven automation from trusting unsigned or malformed provider messages.
6. The explicit split between provider facts here, semantic truth in [`domain`](../../domain/README.md), and persisted projections in [`storage`](../../storage/README.md) lets reviewers inspect small boundary surfaces before trusting automated checkout, CRM, manager-brief, labor, or retail workflows.

## Cross-crate relationships

- [`domain`](../../domain/README.md) owns semantic truth and source provenance. Gingr mappings currently promote into `domain::customer`, `domain::pet`, `domain::entities::ContactChannel`, and `domain::retail` values; source-specific Gingr provenance and source-agnostic snapshots live in [`../../domain/src/source.rs`](../../domain/src/source.rs). Domain service-line guides include [`../../domain/src/boarding/README.md`](../../domain/src/boarding/README.md), [`../../domain/src/daycare/README.md`](../../domain/src/daycare/README.md), [`../../domain/src/grooming/README.md`](../../domain/src/grooming/README.md), [`../../domain/src/training/README.md`](../../domain/src/training/README.md), and [`../../domain/src/retail/README.md`](../../domain/src/retail/README.md).
- [`app`](../../app/src/lib.rs) composes normalized domain facts into workflows. Gingr-sourced facts appear in workflow tests and local smoke data as `domain::source::System::Gingr` and raw payload refs, while workflow modules such as [`../../app/src/checkout_completion.rs`](../../app/src/checkout_completion.rs), [`../../app/src/crm_retention.rs`](../../app/src/crm_retention.rs), [`../../app/src/manager_daily_brief.rs`](../../app/src/manager_daily_brief.rs), [`../../app/src/daily_update.rs`](../../app/src/daily_update.rs), and [`../../app/src/tools.rs`](../../app/src/tools.rs) should consume normalized evidence rather than raw provider payloads.
- [`storage`](../../storage/README.md) owns persisted projections and stable storage codes. It documents the provider/projection boundary in [`../../storage/README.md`](../../storage/README.md) and stores operations/source evidence in [`../../storage/src/operations.rs`](../../storage/src/operations.rs), including Gingr as a core operating-system code and manager daily brief source reference.
- [`docs/integrations/gingr`](../../docs/integrations/gingr/README.md) is the provider documentation and fixture inventory. Use it to trace where endpoint/webhook assumptions came from; keep crate-level behavior grounded in source files and tests rather than marketing claims.
- App shells under [`../../apps`](../../apps) should reach Gingr through application/integration seams. They should not become alternate provider adapters.

## Maintainer notes

- Preserve semantic module paths in prose and code. Prefer `gingr::endpoint::owners_animals::OwnerLookup`, `gingr::mapping::retail::ProductCandidate`, or `domain::source::gingr::Provenance` over vague names such as "owner lookup" or "product" when the boundary matters.
- Add provider endpoint types under [`src/endpoint`](./src/endpoint/mod.rs) when the crate needs to construct a Gingr request. Add DTOs under [`src/dto`](./src/dto/mod.rs) only when the provider shape is documented or fixture-backed enough to deserialize safely.
- Add mapping functions under [`src/mapping`](./src/mapping/mod.rs) when a provider record can be promoted into a named domain candidate. Mapping functions should fail loudly on required provider fields and invalid domain values.
- Do not treat provider ids from `gingr::endpoint::*Id` wrappers as canonical `domain::entities::*Id` values. Keep the relationship in source provenance, mapping candidates, or explicit storage/source references.
- Do not claim live HTTP support until [`src/transport.rs`](./src/transport.rs) implements it; `HttpTransport` currently returns `TransportError::HttpNotImplemented`.
