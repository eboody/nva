# `gingr::mapping`

`gingr::mapping` is the Gingr provider-adapter module that promotes selected provider DTO/response fields into semantic domain candidate values. It owns the first explicit trust-boundary crossing after provider deserialization: required provider fields are checked, provider strings are validated with `domain::*` constructors, and failures become typed mapping errors instead of silent defaults.

This module is not Gingr-as-domain. A mapping candidate such as [`gingr::mapping::retail::ProductCandidate`](./retail.rs) carries source provenance hints like provider ids plus validated `domain::retail` values, but it is still an integration candidate. Domain truth lives in [`domain`](../../../../domain/README.md), durable projection records live in [`storage`](../../../../storage/README.md), and application workflows live in [`app`](../../../../app/README.md).

## Module navigation

Start at [`mod.rs`](./mod.rs). It declares mapping families, defines [`gingr::mapping::Result<T>`](./mod.rs), and owns the shared error vocabulary:

- [`gingr::mapping::ProviderField`](./mod.rs) names the provider fields that mappings currently require or validate: owner name, animal name, retail item name, retail item SKU, retail item category, and retail item active flag.
- [`gingr::mapping::Error`](./mod.rs) distinguishes missing required provider fields from invalid domain values promoted from provider fields.

Mapping family files:

- [`customer.rs`](./customer.rs) maps [`gingr::response::OwnerRecord`](../response.rs) into [`gingr::mapping::customer::ContactCandidate`](./customer.rs). It combines first/last name with `OwnerRecord::display_name`, validates `domain::customer::Name`, optionally validates `domain::customer::Email` and `domain::customer::Phone`, and derives `domain::entities::ContactChannel` as Email, Sms, or Portal based on available contact facts.
- [`pet.rs`](./pet.rs) maps [`gingr::response::AnimalRecord`](../response.rs) into [`gingr::mapping::pet::NameCandidate`](./pet.rs) by requiring and validating `domain::pet::Name` while preserving the Gingr `endpoint::AnimalId`.
- [`retail.rs`](./retail.rs) maps [`gingr::dto::retail::Item`](../dto/retail.rs) into [`gingr::mapping::retail::ProductCandidate`](./retail.rs). It requires item name, SKU, category, and active flag, validates `domain::retail::product::Name` and `domain::retail::Sku`, promotes supported category strings into `domain::retail::product::Category`, and maps the provider `active` flag into `domain::retail::OfferingStatus::Active` or `Inactive` without silently defaulting missing provider evidence.

## Semantic promotion boundary

Mappings should be read as semantic promotion, not field copying:

1. Provider DTOs and response records preserve source facts. [`../dto/retail.rs`](../dto/retail.rs) and [`../response.rs`](../response.rs) are still provider-shaped.
2. Mapping functions decide which provider facts are required and which can stay optional. Missing owner name, animal name, retail item name, retail SKU, retail category, or retail active flag becomes `Error::MissingRequiredProviderField`.
3. Domain constructors enforce semantic constraints. Invalid customer names, emails, phones, pet names, retail product names, SKUs, or retail categories become `Error::InvalidDomainValue` with a `ProviderField` label.
4. Candidate structs keep provider ids (`endpoint::OwnerId`, `endpoint::AnimalId`, `dto::retail::ItemId`) alongside validated domain values. Those ids are source references, not canonical domain ids.
5. Mappings do not write storage records, send messages, mutate Gingr, capture payments, or book reservations. They prepare validated candidate values for app/storage/review code to decide what can be persisted or acted on.

## Source provenance and target concepts

Current mappings have these source-to-target relationships:

- [`customer::contact_candidate`](./customer.rs) reads [`response::OwnerRecord`](../response.rs) fields `id`, `first_name`, `last_name`, `email`, and `cell_phone`. It targets [`domain::customer::Name`](../../../../domain/src/customer.rs), [`domain::customer::Email`](../../../../domain/src/customer.rs), [`domain::customer::Phone`](../../../../domain/src/customer.rs), and [`domain::entities::ContactChannel`](../../../../domain/src/entities.rs).
- [`pet::name_candidate`](./pet.rs) reads [`response::AnimalRecord`](../response.rs) fields `id` and `name`. It targets [`domain::pet::Name`](../../../../domain/src/pet.rs).
- [`retail::product_candidate`](./retail.rs) reads [`dto::retail::Item`](../dto/retail.rs) fields `id`, `name`, `sku`, `category`, and `active`. It targets [`domain::retail::product::Name`](../../../../domain/src/retail/product.rs), [`domain::retail::Sku`](../../../../domain/src/retail/product.rs), [`domain::retail::Product`](../../../../domain/src/retail/product.rs), [`domain::retail::product::Category`](../../../../domain/src/retail/product.rs), and [`domain::retail::OfferingStatus`](../../../../domain/src/retail/product.rs).

The provider source remains visible because each candidate retains the provider id that produced the mapped value. If a workflow needs durable source lineage, compose these candidates with `domain::source` values from [`domain/src/source.rs`](../../../../domain/src/source.rs) or storage/source-reference records rather than treating a Gingr id as a domain id.

## Type/module map

| Concept | Public type/module path | Defined in | Role |
| --- | --- | --- | --- |
| Mapping module registry | `gingr::mapping` | [`mod.rs`](./mod.rs) | Declares mapping families and shared result/error types. |
| Mapping result alias | `gingr::mapping::Result<T>` | [`mod.rs`](./mod.rs) | Standard result shape for provider-to-domain promotion. |
| Provider field labels | `gingr::mapping::ProviderField` | [`mod.rs`](./mod.rs) | Names provider fields used in missing/invalid mapping errors. |
| Mapping error | `gingr::mapping::Error` | [`mod.rs`](./mod.rs) | Reports missing required provider fields and invalid promoted domain values. |
| Customer contact candidate | `gingr::mapping::customer::ContactCandidate` | [`customer.rs`](./customer.rs) | Holds provider owner id plus validated customer/contact values. |
| Customer mapping function | `gingr::mapping::customer::contact_candidate` | [`customer.rs`](./customer.rs) | Promotes `gingr::response::OwnerRecord` into a contact candidate. |
| Pet name candidate | `gingr::mapping::pet::NameCandidate` | [`pet.rs`](./pet.rs) | Holds provider animal id plus validated pet name. |
| Pet mapping function | `gingr::mapping::pet::name_candidate` | [`pet.rs`](./pet.rs) | Promotes `gingr::response::AnimalRecord` into a pet-name candidate. |
| Retail product candidate | `gingr::mapping::retail::ProductCandidate` | [`retail.rs`](./retail.rs) | Holds provider item id plus validated retail product, name, and offering status. |
| Retail mapping function | `gingr::mapping::retail::product_candidate` | [`retail.rs`](./retail.rs) | Promotes `gingr::dto::retail::Item` into retail domain candidate values. |
| Owner source record | `gingr::response::OwnerRecord` | [`../response.rs`](../response.rs) | Provider-shaped owner record used by customer mapping. |
| Animal source record | `gingr::response::AnimalRecord` | [`../response.rs`](../response.rs) | Provider-shaped animal record used by pet mapping. |
| Retail source DTO | `gingr::dto::retail::Item` | [`../dto/retail.rs`](../dto/retail.rs) | Provider-shaped retail item record used by retail mapping. |
| Customer target values | `domain::customer::{Name, Email, Phone}` | [`../../../../domain/src/customer.rs`](../../../../domain/src/customer.rs) | Validated customer/contact semantic values. |
| Pet target value | `domain::pet::Name` | [`../../../../domain/src/pet.rs`](../../../../domain/src/pet.rs) | Validated pet name semantic value. |
| Retail target values | `domain::retail::{Product, Sku, OfferingStatus}`, `domain::retail::product::{Name, Category}` | [`../../../../domain/src/retail/product.rs`](../../../../domain/src/retail/product.rs) | Validated retail product and offering semantic values. |

## Labor-cost and automation role

`gingr::mapping` reduces labor cost by turning source-data normalization into reviewable code:

1. Customer and pet mappings validate names and contact fields once at the provider boundary, reducing manual cleanup before booking triage, daily updates, CRM follow-up, or manager review can use normalized facts.
2. Retail mapping turns catalog item JSON into `domain::retail` candidate values so retail recommendations, checkout review, and inventory exception triage can depend on semantic product/category/status values instead of raw provider strings.
3. Typed mapping errors let automation route bad source records to exception queues. A missing owner name or unsupported retail category can be reported as a specific provider-field problem rather than becoming a vague downstream workflow failure.
4. Provider ids are retained as source handles, so reviewers can trace a candidate back to Gingr without confusing that id with a canonical domain entity id.
5. Because mappings are side-effect-free, maintainers can test and review source normalization independently before any app shell or storage projection uses the result.

## Cross-crate relationships

- [`integrations/gingr`](../../README.md) is the crate-level guide for the provider adapter. It describes how mapping follows config, endpoint request construction, transport, response, webhook, and DTO handling.
- [`gingr::dto`](../dto/README.md) owns documented provider DTO payloads and provider-surface gaps. Retail mapping currently consumes [`dto::retail::Item`](../dto/retail.rs).
- [`gingr::response`](../response.rs) owns general provider response records. Customer and pet mappings currently consume `OwnerRecord` and `AnimalRecord` from that file.
- [`gingr::endpoint`](../endpoint/README.md) owns provider request shapes and provider id wrappers. Mapping candidates preserve `endpoint::OwnerId` and `endpoint::AnimalId` only as provider/source ids.
- [`domain`](../../../../domain/README.md) owns semantic business truth. Related target files include [`domain/src/customer.rs`](../../../../domain/src/customer.rs), [`domain/src/pet.rs`](../../../../domain/src/pet.rs), [`domain/src/entities.rs`](../../../../domain/src/entities.rs), and [`domain/src/retail/product.rs`](../../../../domain/src/retail/product.rs). The retail domain guide is [`domain/src/retail/README.md`](../../../../domain/src/retail/README.md).
- [`domain::source`](../../../../domain/src/source.rs) owns source-system/provenance vocabulary. Use it when a mapped candidate must become durable evidence or workflow lineage.
- [`storage`](../../../../storage/README.md) owns durable projections and stable storage codes. Service-line storage mappings are documented in [`storage/src/service_line/README.md`](../../../../storage/src/service_line/README.md), and operations/source evidence lives in [`storage/src/operations.rs`](../../../../storage/src/operations.rs).
- [`app`](../../../../app/README.md) composes normalized facts into reviewable workflows such as booking triage, checkout completion, CRM retention, daily updates, and manager daily brief. App code should consume mapped domain/source facts through proper ports rather than parsing raw Gingr payloads.
- [`docs/integrations/gingr`](../../../../docs/integrations/gingr/README.md) is the provider documentation/fixture inventory to consult before extending a mapping to a new endpoint or DTO.

## Maintainer notes

- Preserve semantic module paths in prose and code. Prefer `gingr::mapping::customer::ContactCandidate`, `gingr::mapping::retail::ProductCandidate`, `domain::customer::Name`, and `domain::retail::product::Category` over flattened labels when a boundary matters.
- Add mappings only when source fields are documented or fixture-backed enough to promote safely. If a provider field is missing or ambiguous, return a typed `gingr::mapping::Error` or document a DTO/provider gap instead of defaulting into business meaning.
- Keep meaningful transformations named. Validation, category normalization, contact-channel derivation, and active/status mapping are semantic promotion steps; do not hide new promotion rules behind casual `.into()` chains.
- Keep side effects out of this module. Storage writes, app workflow decisions, live messages, provider mutation, and payment behavior belong in storage/app/shell layers after mapping has produced reviewable candidate values.
