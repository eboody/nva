# `gingr::endpoint`

`gingr::endpoint` is the Gingr provider-adapter module that turns named Rust request values into Gingr API method/path/parameter parts. It owns the provider-facing request vocabulary: [`Request`](./mod.rs), [`Method`](./mod.rs), [`Path`](./mod.rs), date and pagination scalars, provider id wrappers, and endpoint-family modules for catalog, reference data, reservations, owners/animals, commerce/retail, report-card files, and labor operations.

This module is request construction, not domain truth and not live HTTP. A request type such as `gingr::endpoint::reservations::BackOfHouse` or `gingr::endpoint::owners_animals::Owner` knows how to produce the Gingr path and parameters for one provider endpoint. [`gingr::transport`](../transport.rs) turns those request parts into transport input and applies API-key injection/redaction. [`gingr::response`](../response.rs), [`gingr::dto`](../dto/mod.rs), and [`gingr::mapping`](../mapping/mod.rs) own provider response envelopes, documented DTO records, and promotion into `domain::*` candidates.

## Module navigation

Start at [`mod.rs`](./mod.rs). It declares all endpoint families, re-exports `gingr::endpoint::Reservations`, defines shared validation errors, and provides the `Request::request_parts` adapter into [`transport::RequestParts`](../transport.rs).

Endpoint families:

- [`catalog.rs`](./catalog.rs) is a provider-surface inventory rather than a request builder module. It lists documented endpoint names and semantic mapping gaps for `retail`, `training`, and `grooming`.
- [`reference_data.rs`](./reference_data.rs) builds small read-only reference endpoints: locations, species, breeds, vets, temperaments, immunization types, and animal immunizations.
- [`reservations.rs`](./reservations.rs) builds reservation search, reservation-type, reservation-widget, checked-in/range reservations, back-of-house, services-by-type, and reservation-by-animal/owner requests.
- [`owners_animals.rs`](./owners_animals.rs) builds owner/animal list and lookup requests, form/custom-field lookups, and animal feeding/medication requests. Sensitive phone/email/custom-field search values are marked for redaction through `Request::sensitive_parameter_names`.
- [`commerce_retail.rs`](./commerce_retail.rs) builds retail item, subscription, transaction, and invoice requests. It also marks `Transaction` responses as `ResponseSensitivity::PaymentSensitive`.
- [`report_cards_files.rs`](./report_cards_files.rs) builds `report_card_files` requests with optional days, limit, and location filters.
- [`labor_ops.rs`](./labor_ops.rs) builds `timeclock_report` requests for labor/operations evidence.

## Shared request vocabulary

[`mod.rs`](./mod.rs) is intentionally small and semantic:

- `gingr::endpoint::Request` is the common trait every endpoint request implements. It exposes `method`, `path`, `parameters`, and optional `sensitive_parameter_names`; `request_parts` packages those values for [`gingr::transport::RequestParts`](../transport.rs).
- `gingr::endpoint::Method` is the local `Get`/`Post` enum used by endpoint requests before transport conversion.
- `gingr::endpoint::Path` wraps static provider paths so transport receives a typed path instead of a loose string.
- `gingr::endpoint::Date`, `IsoDate`, and `DateRange` validate provider date inputs. `DateRange::new` rejects reversed ranges and ranges longer than the 30-day reservations window encoded in source.
- `gingr::endpoint::Limit` rejects zero limits.
- `gingr::endpoint::{AnimalId, OwnerId, ReservationId, LocationId, SpeciesId, FormId, ReferenceId}` are provider-side numeric wrappers. They should not be treated as canonical domain entity ids.
- `gingr::endpoint::Error` and `Result<T>` name endpoint-construction failures such as invalid dates, reversed ranges, empty text, missing builder parameters, legacy commerce date boundaries, invalid pagination, and invalid subscription bill days.

## Endpoint families and labor-cost workflows

### Catalog surface

[`catalog.rs`](./catalog.rs) exposes `exported_read_endpoint_names` and `semantic_mapping_gaps`. It helps maintainers compare the documented Gingr surface with what this crate has typed enough to request or map. That reduces review labor by keeping unsupported retail/training/grooming gaps explicit instead of scattering TODOs or pretending a DTO exists.

### Reference data

[`reference_data.rs`](./reference_data.rs) contains `GetLocations`, `GetSpecies`, `GetBreeds`, `GetTemperaments`, `GetVets`, `GetVetsBuilder`, `GetImmunizationTypes`, and `GetAnimalImmunizations`.

Reference-data requests support labor-cost workflows by normalizing provider-side lookup tables before downstream automation uses them: location filters for manager briefs and timeclock reports, species/breed/vet/immunization context for animal-care review, and immunization lookups for safer exception triage. The source only builds these requests; response normalization happens through [`response.rs`](../response.rs), [`dto`](../dto/mod.rs), and mapping modules when supported.

### Reservations and service availability

[`reservations.rs`](./reservations.rs) contains nested `gingr::endpoint::reservations::reservation` types for reservation metadata/search filters plus the top-level reservation request family:

- `gingr::endpoint::reservations::reservation::{TypeId, Types, TypesBuilder}` for `reservation_types`.
- `gingr::endpoint::reservations::reservation::{WidgetData, WidgetDataBuilder}` for `reservation_widget_data`.
- `gingr::endpoint::reservations::reservation::{SearchFilters, SearchFiltersBuilder}` for shared date/type/animal/status/limit filters.
- `gingr::endpoint::Reservations` and `gingr::endpoint::reservations::Builder` for checked-in or bounded-date reservation searches.
- `gingr::endpoint::reservations::RestrictTo` for `pending_requests`, `currently_checked_in`, `future`, `past`, and `wait_listed` filters.
- `gingr::endpoint::reservations::by::{Animal, AnimalBuilder, Owner, OwnerBuilder}` for `reservations_by_animal` and `reservations_by_owner`; the source caveat says these endpoint results are scoped to the location where the API user is logged in.
- `gingr::endpoint::reservations::{MinutesFuture, BackOfHouse, BackOfHouseBuilder}` for `back_of_house`.
- `gingr::endpoint::reservations::GetServicesByType` for service lookup by reservation type and optional location.

These requests can feed manager/admin labor reduction around occupancy, check-in exception triage, wait-list/pending-request review, service availability, and back-of-house planning. They do not, by themselves, decide capacity or scheduling policy; those semantics belong in domain/application modules after provider facts are mapped.

### Owners, animals, forms, and care facts

[`owners_animals.rs`](./owners_animals.rs) contains:

- `ProviderWhereClause`, `Owners`, `OwnersBuilder`, `Animals`, and `AnimalsBuilder` for provider `params[...]` filtered owner/animal lists.
- `SensitiveLookup`, `OwnerLookup`, and `Owner` for `owner` lookup by owner id, animal id, reservation id, phone, or email. Phone/email lookups are sensitive parameters.
- `FormKind` and `Form` for owner/animal form retrieval.
- `custom_field::{Name, Search, SearchBuilder}` for `custom_field_search`; `search` is marked sensitive.
- `AnimalCareInfo` for feeding and medication info.

These requests support fewer front-desk/admin handoffs by locating owners/animals, retrieving care instructions, and isolating sensitive search values so request logs can be reviewed safely. They also provide source evidence for customer/pet normalization work in [`mapping::customer`](../mapping/customer.rs) and [`mapping::pet`](../mapping/pet.rs) when response records are available.

### Commerce and retail

[`commerce_retail.rs`](./commerce_retail.rs) contains:

- `get::AllRetailItems` for `get_all_retail_items`.
- `get::{SubscriptionId, Subscription}` for `get_subscription`.
- `get::{BillDayOfMonth, PackageId, SubscriptionPagination, Subscriptions, SubscriptionsBuilder}` for `get_subscriptions`.
- `list::{Transactions, TransactionsBuilder}` for legacy `list_transactions`; its builder rejects dates on or after the documented 2019-08-01 cutover.
- `list::{InvoicePagination, Invoices, InvoicesBuilder}` for `list_invoices`; its builder rejects pre-cutover invoice dates and validates the provider-specific page/per-page rule.
- `TransactionId`, `Transaction`, and `ResponseSensitivity::PaymentSensitive` for the `transaction` endpoint.

Commerce/retail requests support checkout, invoice reconciliation, subscription review, and retail catalog normalization. The module deliberately encodes provider boundary quirks, including the 2019-08-01 transaction/invoice split and payment-sensitive transaction detail. Retail product promotion is handled in [`mapping::retail`](../mapping/retail.rs) from [`dto::retail::Item`](../dto/retail.rs), not directly by these request builders.

### Report-card files

[`report_cards_files.rs`](./report_cards_files.rs) contains `ReportCardFiles` and `ReportCardFilesBuilder` for `report_card_files` with optional `number_days`, `limit`, and `location_id` parameters.

Report-card file requests can support care-summary follow-up, exception review, and manager visibility into recent customer-facing care artifacts. The endpoint module only builds the request; file interpretation, customer communication decisions, and durable storage policy belong outside this module.

### Labor operations

[`labor_ops.rs`](./labor_ops.rs) contains `UserId`, `TimeclockReport`, and `TimeclockReportBuilder` for `timeclock_report`. The builder requires `start_date`, `end_date`, and `location_id`, and can include deleted users, currently clocked-in users, and one or more `user_ids[]` filters.

This is the direct labor-cost endpoint family. It can support timeclock audit, payroll exception review, location-level staffing reports, clocked-in-now checks, and manager daily labor evidence. It still only produces request parameters; app/storage code must decide which labor facts are persisted, reconciled, or escalated.

## Response and transport boundaries

`gingr::endpoint` request types return parameter lists; they do not deserialize HTTP bodies. Response-related public types live in [`../response.rs`](../response.rs): `gingr::response::Raw`, `HttpStatus`, `Envelope<T>`, `OwnerRecord`, `AnimalRecord`, `ReservationRecord`, and `ReferenceRecord`. Provider DTO modules live under [`../dto`](../dto/mod.rs), including [`dto::retail::Item`](../dto/retail.rs). Promotion into semantic domain candidates lives under [`../mapping`](../mapping/mod.rs).

That split is important for safe automation: endpoint builders make request construction reviewable, transport redacts secrets and captures request shape, response/DTO code preserves provider facts, and mapping code decides what can become domain evidence.

## Type/module map

| Concept | Public type/module path | Defined in | Role |
| --- | --- | --- | --- |
| Endpoint module registry | `gingr::endpoint` | [`mod.rs`](./mod.rs) | Declares endpoint families and shared request vocabulary. |
| Endpoint error/result | `gingr::endpoint::{Error, Result}` | [`mod.rs`](./mod.rs) | Names request-construction validation failures. |
| Request trait | `gingr::endpoint::Request` | [`mod.rs`](./mod.rs) | Converts typed requests into method/path/parameter parts. |
| Method/path primitives | `gingr::endpoint::{Method, Path}` | [`mod.rs`](./mod.rs) | Provider HTTP method and static path wrappers. |
| Date/range primitives | `gingr::endpoint::{Date, IsoDate, DateRange}` | [`mod.rs`](./mod.rs) | Validated provider date values and bounded reservation ranges. |
| Limit primitive | `gingr::endpoint::Limit` | [`mod.rs`](./mod.rs) | Non-zero provider limit value. |
| Provider ids | `gingr::endpoint::{AnimalId, OwnerId, ReservationId, LocationId, SpeciesId, FormId, ReferenceId}` | [`mod.rs`](./mod.rs) | Numeric Gingr identifiers scoped to provider requests. |
| Catalog inventory | `gingr::endpoint::catalog::{exported_read_endpoint_names, semantic_mapping_gaps}` | [`catalog.rs`](./catalog.rs) | Documents supported read endpoint names and mapping gaps. |
| Reference requests | `gingr::endpoint::reference_data::{GetLocations, GetSpecies, GetBreeds, GetVets, GetTemperaments, GetImmunizationTypes, GetAnimalImmunizations}` | [`reference_data.rs`](./reference_data.rs) | Builds reference-data requests. |
| Reservation metadata | `gingr::endpoint::reservations::reservation::{TypeId, Types, WidgetData, SearchFilters}` | [`reservations.rs`](./reservations.rs) | Builds reservation type/widget/filter request pieces. |
| Reservation search | `gingr::endpoint::Reservations` | [`reservations.rs`](./reservations.rs) | Builds checked-in or date-range `reservations` requests. |
| Reservation by entity | `gingr::endpoint::reservations::by::{Animal, Owner}` | [`reservations.rs`](./reservations.rs) | Builds reservations-by-animal/owner requests. |
| Back-of-house/service lookup | `gingr::endpoint::reservations::{BackOfHouse, GetServicesByType, MinutesFuture, RestrictTo}` | [`reservations.rs`](./reservations.rs) | Builds operations/service availability requests. |
| Owner/animal lists | `gingr::endpoint::owners_animals::{ProviderWhereClause, Owners, Animals}` | [`owners_animals.rs`](./owners_animals.rs) | Builds provider-filtered owner/animal list requests. |
| Owner lookup | `gingr::endpoint::owners_animals::{SensitiveLookup, OwnerLookup, Owner}` | [`owners_animals.rs`](./owners_animals.rs) | Builds owner lookup requests and marks sensitive phone/email parameters. |
| Forms/custom fields | `gingr::endpoint::owners_animals::{FormKind, Form}`, `gingr::endpoint::owners_animals::custom_field::{Name, Search}` | [`owners_animals.rs`](./owners_animals.rs) | Builds form and custom-field search requests. |
| Animal care info | `gingr::endpoint::owners_animals::AnimalCareInfo` | [`owners_animals.rs`](./owners_animals.rs) | Builds feeding/medication info requests. |
| Retail item request | `gingr::endpoint::commerce_retail::get::AllRetailItems` | [`commerce_retail.rs`](./commerce_retail.rs) | Builds retail catalog request. |
| Subscription requests | `gingr::endpoint::commerce_retail::get::{SubscriptionId, Subscription, BillDayOfMonth, PackageId, SubscriptionPagination, Subscriptions}` | [`commerce_retail.rs`](./commerce_retail.rs) | Builds subscription lookup/list requests. |
| Transaction/invoice lists | `gingr::endpoint::commerce_retail::list::{Transactions, InvoicePagination, Invoices}` | [`commerce_retail.rs`](./commerce_retail.rs) | Builds commerce list requests with legacy date/pagination validation. |
| Transaction detail | `gingr::endpoint::commerce_retail::{TransactionId, Transaction, ResponseSensitivity}` | [`commerce_retail.rs`](./commerce_retail.rs) | Builds transaction detail request and labels payment-sensitive responses. |
| Report-card files | `gingr::endpoint::report_cards_files::{ReportCardFiles, ReportCardFilesBuilder}` | [`report_cards_files.rs`](./report_cards_files.rs) | Builds report-card file requests. |
| Labor timeclock | `gingr::endpoint::labor_ops::{UserId, TimeclockReport, TimeclockReportBuilder}` | [`labor_ops.rs`](./labor_ops.rs) | Builds timeclock report requests. |
| Transport request parts | `gingr::transport::RequestParts` | [`../transport.rs`](../transport.rs) | Receives endpoint method/path/parameters and applies API-key/redaction behavior. |
| Raw response records | `gingr::response::{Raw, Envelope, OwnerRecord, AnimalRecord, ReservationRecord, ReferenceRecord}` | [`../response.rs`](../response.rs) | Deserializes provider response facts outside the endpoint module. |

## Cross-crate relationships

- [`integrations/gingr`](../../README.md) is the crate-level guide. It explains how `endpoint` fits with config, transport, response, webhook, DTO, and mapping modules.
- [`gingr::transport`](../transport.rs) consumes `gingr::endpoint::Request::request_parts`, injects API keys, redacts sensitive parameters, and provides the transport seam. `HttpTransport` is not implemented in this slice; tests use mock transport.
- [`gingr::response`](../response.rs), [`gingr::dto`](../dto/mod.rs), and [`gingr::mapping`](../mapping/mod.rs) are the next boundary after request construction. `mapping::customer`, `mapping::pet`, and `mapping::retail` promote supported provider records into domain candidates.
- [`domain`](../../../../domain/README.md) owns semantic business truth. Related domain guides include [`domain::boarding`](../../../../domain/src/boarding/README.md), [`domain::daycare`](../../../../domain/src/daycare/README.md), [`domain::grooming`](../../../../domain/src/grooming/README.md), [`domain::training`](../../../../domain/src/training/README.md), and [`domain::retail`](../../../../domain/src/retail/README.md).
- [`storage`](../../../../storage/README.md) owns durable projections and source evidence. Its service-line guide at [`storage::service_line`](../../../../storage/src/service_line/README.md) documents how normalized service-line records relate to provider evidence.
- [`app`](../../../../app/src/lib.rs) composes normalized facts into workflows such as checkout completion, CRM retention, manager daily brief, daily updates, and tools. App modules should not assemble raw Gingr query strings directly when these endpoint request types can preserve provider semantics.
- [`docs/integrations/gingr`](../../../../docs/integrations/gingr/README.md) is the provider documentation/fixture inventory. Use it when deciding whether a new endpoint builder, DTO, or mapping is source-backed.
- [`apps`](../../../../apps) contains app shells; they should reach Gingr through app/integration seams rather than duplicating endpoint construction.

## Maintainer notes

- Preserve semantic module paths in code and prose. Prefer `gingr::endpoint::reservations::reservation::SearchFilters`, `gingr::endpoint::owners_animals::OwnerLookup`, and `gingr::endpoint::commerce_retail::list::Invoices` over flattened names when the provider family matters.
- Add new request builders under the endpoint-family file that matches the provider concept. If the provider concept is unsupported or only documented as a gap, record that fact in [`catalog.rs`](./catalog.rs) or the relevant DTO/mapping gap instead of inventing response types.
- Keep provider ids as `gingr::endpoint::*Id` values until a mapping layer can attach source provenance and validate domain meaning.
- Mark sensitive query parameters through `Request::sensitive_parameter_names` whenever a request can carry phone, email, payment, custom-field, or other human-identifying lookup values.
- Do not claim a workflow is automated just because an endpoint can request the source data. This module reduces labor by making data collection typed and reviewable; domain, app, and storage modules decide which facts are normalized, persisted, escalated, or acted on.
