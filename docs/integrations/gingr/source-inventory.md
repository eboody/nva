# Gingr source inventory for source-of-record and BI read models

Inventory date: 2026-06-16

Scope: static repository inspection only. This inventory uses current source and
repo docs under these areas:

- `integrations/gingr`
- `domain`
- `storage`
- `app`
- `docs/integrations/gingr`

No real Gingr credentials, tenant access, or live data were used.

## Why this matters

Tyler reported that Gingr is already being used as a de facto database. He also
reported that an existing messy code path pulls Gingr data into BI-facing types.
In this repo, Gingr should therefore be treated as a provider/source-data
boundary, not as a clean domain model.

The desired lane is:

1. Preserve raw Gingr facts with provenance.
2. Promote them through explicit semantic mappings.
3. Emit data-quality issues when source facts are missing or ambiguous.
4. Project validated facts into BI/read-model shapes.

The post-refactor reservation/stay boundary is documented in
`docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md`:

```text
Gingr DTO -> Gingr snapshot -> source-agnostic reservation snapshot -> analytics stay fact -> future workflow validators
```

The repo now contains the first source-contract slice for reservation/stay
evidence: source-agnostic provenance, source record identities, reservation
snapshots, data-quality issues, and `analytics::stay::Fact` projection. That
slice is fixture/test oriented; it is not a live Gingr or BI database
integration.

## Current boundary posture

`integrations/gingr` owns provider DTOs, endpoint request builders, redacted
transport setup, webhook verification, raw response wrappers, and narrow
mappers into domain/app concepts.

`domain` owns semantic entities and operations vocabulary. It already recognizes
Gingr as a portal/source context and recognizes BI/data-quality concepts at an
operations-discovery level.

The source contract now separates `source::gingr::*` adapter vocabulary from the
source-agnostic core paths used by data quality and analytics:

- Gingr adapter: `source::gingr::Provenance`,
  `source::gingr::ProviderRecordId`, `source::gingr::ProviderStatus`, and
  `source::gingr::reservation::Snapshot`.
- Core source contract: `source::Provenance`, `source::record::*`,
  `source::reservation::Snapshot`, `source::reservation::Status`, and
  `source::reservation::Assumption`.
- Analytics projection: `analytics::stay::Fact::project_from_source_reservation`
  consumes the source-agnostic snapshot, not the Gingr adapter snapshot.

`storage` owns storage-shaped operation records and codecs. It mirrors the
operations vocabulary for `TechnologyEcosystemRecord`, including Gingr,
API/webhook/export/warehouse/BI data-access patterns, and BI/data-lake adjacent
systems.

`app` owns use-case/tool ports. It exposes a provider-agnostic
`tools::portal::Lookup` port with Gingr as one provider, but it does not yet
expose BI extraction or projection ports.

`docs/integrations/gingr` contains the public Gingr support-site corpus,
endpoint catalog, source audit, webhook fixtures, and SDK design notes used to
build the current provider boundary.

## Local documentation corpus

Canonical local docs:

- `docs/integrations/gingr/source-audit.md`
- `docs/integrations/gingr/sdk-endpoint-catalog.md`
- `docs/integrations/gingr/sdk-architecture.md`
- `docs/integrations/gingr/sdk-readiness-review.md`
- `docs/integrations/gingr/sdk-webhooks.md`
- `docs/integrations/gingr/sdk-customer-portal-js.md`
- `docs/integrations/gingr/service-domain-provider-surfaces.md`
- `docs/integrations/gingr/adapter-boundary-and-labor-source-expansion.md`
- `docs/integrations/gingr/fixtures/webhooks/`
- `docs/integrations/gingr/articles/`
- `docs/integrations/gingr/manifest.json`

Important doc facts already captured:

- Public API base URL pattern is `https://{your_app}.gingrapp.com`.
- API requests use a user-account-specific `key` parameter.
- API responses are documented as HTTPS JSON responses.
- Gingr describes the public API as read-only.
- The repo still excludes side-effecting functions from the v0 read SDK.
- Excluded side-effecting functions are `quick_checkin` and `receive_call`.
- Public response schemas are partial and example-oriented.
- Many response fields should remain provider/raw until mapped deliberately.
- Some reservation endpoints depend on the API user's logged-in location.
- Other endpoints accept explicit `location_id` filters.
- The docs include webhook event, signature, retry, and fixture information.
- The docs do not provide a production replay or sandbox fixture suite.

## Exported Gingr read endpoint builders

`integrations/gingr/src/endpoint/catalog.rs` exports these read endpoint names.

Reference data:

- `get_locations`
- `get_species`
- `get_breeds`
- `get_vets`
- `get_temperaments`
- `get_immunization_types`
- `get_animal_immunizations`

BI/source relevance: location, species, breed, vet, temperament, vaccine, and
immunization dimensions or compliance facts.

Reservation and stay:

- `reservation_types`
- `get_services_by_type`
- `reservation_widget_data`
- `reservations`
- `reservations_by_animal`
- `reservations_by_owner`
- `back_of_house`

BI/source relevance: stay/service utilization, occupancy, check-in/check-out,
dashboard counts, service-type/add-on facts, and operational whiteboard state.

Owners, animals, and forms:

- `owner`
- `owners`
- `animals`
- `forms_get_form`
- `custom_field_search`
- `get_feeding_info`
- `get_medication_info`

BI/source relevance: customer and pet dimensions, owner-pet relationships,
custom fields, care instructions, medication, and feeding profile facts. These
surfaces can contain high-PII data and require redaction/quarantine.

Commerce, retail, revenue, and subscriptions:

- `get_all_retail_items`
- `list_transactions`
- `transaction`
- `list_invoices`
- `get_subscription`
- `get_subscriptions`

BI/source relevance: revenue, invoices/estimates, payments/transactions,
subscriptions/packages, and retail-product dimensions. Payment-sensitive
responses require quarantine.

Labor, operations, and files:

- `timeclock_report`
- `report_card_files`

BI/source relevance: labor/staffing facts and report-card media/file activity.
These can feed labor/revenue, staffing, and customer communication dashboards.

Explicitly excluded from the read-only SDK surface:

- `GET /api/v1/quick_checkin`
  - Checks in pets and may create reservations.
- `POST /api/v1/receive_call`
  - Records and notifies an incoming call.

## Endpoint implementation facts useful for source snapshots

### Reservations and stay-like source facts

Implemented in `integrations/gingr/src/endpoint/reservations.rs`:

- `Reservations`: `POST /api/v1/reservations`.
  - Supports `checked_in=true` or a `DateRange`.
  - The builder enforces the documented 30-day maximum range.
  - Optional `location_id` is modeled.
- `reservation::Types`: `GET /api/v1/reservation_types`.
  - Supports optional `id` and `active_only`.
- `reservation::WidgetData`: `GET /api/v1/reservation_widget_data`.
  - Requires a `YYYY-MM-DD` `timestamp`.
  - Useful for dashboard count checks.
- `by::Animal`: `POST /api/v1/reservations_by_animal`.
  - Requires animal ID.
  - Supports `restrict_to`, ISO date filters, type filters, animal filters,
    cancelled/confirmed/completed flags, and limit.
- `by::Owner`: `POST /api/v1/reservations_by_owner`.
  - Same filter shape as `by::Animal`, keyed by owner ID.
- `BackOfHouse`: `GET /api/v1/back_of_house`.
  - Requires `location_id`.
  - Supports reservation type IDs, `mins_future`, and `full_day`.
  - Docs describe checking-in/checking-out arrays, reservations, owner,
    animal, type, timestamps, run/area, belonging counts, status, and event
    time.
- `GetServicesByType`: `GET /api/v1/get_services_by_type`.
  - Keyed by reservation type and optional location.

BI/source facts to preserve:

- Provider reservation ID.
- Owner, animal, reservation type, service type, and location IDs.
- Status and lifecycle timestamps.
- Checked-in, cancelled, confirmed, and completed facts.
- Add-on services, whiteboard run/area facts, and dashboard counts.
- Estimate, invoice, payment, and deposit references when present.
- Exact pull window, endpoint name, request parameters, extraction batch, and
  pulled-at timestamp.
- API-user/location scope caveat because different endpoints can expose
different visibility.

Current gap: `response::ReservationRecord` only models provider reservation ID,
optional owner ID, optional animal ID, raw status, and unknown fields. The
domain crate now has a source-contract slice for Gingr reservation snapshots,
source-agnostic reservation snapshots, data-quality issues, and
`analytics::stay::Fact`; however, the integration crate has not yet wired the
public Gingr response DTO into that source-contract promotion path, and revenue
or utilization projections remain future slices.

### Owner/customer and animal/pet source facts

Implemented in `integrations/gingr/src/endpoint/owners_animals.rs` and
`integrations/gingr/src/response.rs`:

- `Owners`: `GET /api/v1/owners`.
  - Supports arbitrary provider where clauses as `params[...]`.
- `Animals`: `GET /api/v1/animals`.
  - Uses the same arbitrary provider where-clause boundary.
- `Owner`: `GET /api/v1/owner`.
  - One-of lookup by owner ID, animal ID, reservation ID, phone, or email.
  - Phone and email are modeled as `SensitiveLookup`.
- `Form`: `GET /forms/get_form`.
  - Supports `owner_form` and `animal_form`.
- `custom_field::Search`: `GET /api/v1/custom_field_search`.
  - Uses owner/animal form ID, field name, and sensitive search text.
- `AnimalCareInfo`: `GET /api/v1/get_feeding_info` or
  `GET /api/v1/get_medication_info` by animal ID.
- `response::OwnerRecord`:
  - Provider owner ID, first name, last name, email, cell phone, and unknown
    fields.
- `response::AnimalRecord`:
  - Provider animal ID, optional owner ID, name, species, birthday, and unknown
    fields.

Existing semantic mappings:

- `mapping::customer::contact_candidate` promotes a provider owner record into
  a contact candidate with provider owner ID, `customer::Name`, optional
  `customer::Email`, and optional `customer::Phone`.
- `mapping::pet::name_candidate` promotes a provider animal record into a
  pet-name candidate with provider animal ID and `pet::Name`.

BI/source facts to preserve:

- Stable provider owner and animal IDs.
- Owner-pet links.
- Customer contact facts.
- Pet name, species, and birthday.
- Care instructions, medications, feeding profile, custom fields, and form
  definitions.
- Unknown provider fields, while enforcing redaction/quarantine.

Current gaps:

- Owner and animal mappers promote only narrow contact/name facts.
- They do not build full `domain::entities::Customer` or
  `domain::entities::Pet`.
- Missing semantics include location, preferred contact, portal account, pet
  sex, spay/neuter status, temperament, and care profile.
- There is no owner/pet source snapshot.
- There is no dedupe/merge handling.
- There is no explicit source-data issue for duplicate customers, incomplete
  pet profiles, missing vaccine data, or ambiguous owner-pet relationships.

### Reference data and care compliance

Implemented in `integrations/gingr/src/endpoint/reference_data.rs`:

- `GetLocations`: `GET /api/v1/get_locations`.
- Simple reference endpoints for species, breeds, and temperaments.
- `GetVets`: `GET /api/v1/get_vets`.
  - Optional `vetFlag` includes full vet information.
- `GetImmunizationTypes`: `GET /api/v1/get_immunization_types`.
  - Keyed by species ID.
- `GetAnimalImmunizations`: `GET /api/v1/get_animal_immunizations`.
  - Keyed by animal ID.

BI/source facts to preserve:

- Location dimension.
- Species, breed, temperament, vet, and immunization type dimensions.
- Per-animal immunization records.
- Vaccine compliance and pre-arrival workflow gating facts.

Current gaps:

- Only generic `response::ReferenceRecord` exists for reference records.
- There are no typed location, species, breed, vet, or immunization DTOs.
- There is no mapping into `domain::entities::Location` or pet vaccination
  concepts.
- There is no compliance/data-quality projection.

### Commerce, retail, revenue, and subscriptions

Implemented in `integrations/gingr/src/endpoint/commerce_retail.rs` and
`integrations/gingr/src/dto/retail.rs`:

- `get::AllRetailItems`: `GET /api/v1/get_all_retail_items`.
- `list::Transactions`: `GET /api/v1/list_transactions`.
  - Builder enforces the documented pre-2019-08-01 transaction boundary.
- `list::Invoices`: `GET /api/v1/list_invoices`.
  - Builder enforces the on/after-2019-08-01 invoice boundary.
  - Builder enforces paired `per_page` plus result-offset-style `page`.
- `Transaction`: `POST /api/v1/transaction`.
  - Response is marked `PaymentSensitive`.
- `get::Subscription`: `GET /api/v1/get_subscription` by subscription ID.
- `get::Subscriptions`: `GET /api/v1/get_subscriptions`.
  - Supports deleted flag, bill day, owner, limit/offset, location, and package
    filters.
- `dto::retail::Item`:
  - Provider item ID, optional name, optional SKU, optional category, optional
    active flag, optional quantity on hand, and unknown fields.

Existing semantic mapping:

- `mapping::retail::product_candidate` promotes a retail item into provider item
  ID, retail product name, domain retail product/SKU/category, and active or
  inactive offering status.
- The mapper deliberately does not infer inventory policy from
  `quantity_on_hand` because the public docs do not define stock semantics.

BI/source facts to preserve:

- Invoice/estimate ID.
- Transaction/payment ID.
- Owner/reservation links when present.
- Dates, closed/open/completed states, amounts, subscriptions, packages, retail
  SKU/category/status, and cutover provenance.
- Payment-sensitive raw payload references for authorized projection jobs.

Current gaps:

- There are no invoice, transaction, payment, subscription, package, or revenue
  DTOs beyond request builders and retail-item DTOs.
- No mapper exists from Gingr invoice/transaction facts into
  `domain::payment::Deposit` or BI revenue facts.
- Historical cutover behavior needs explicit provenance before BI backfills.

### Labor operations and report-card files

Implemented in `integrations/gingr/src/endpoint/labor_ops.rs` and
`integrations/gingr/src/endpoint/report_cards_files.rs`:

- `TimeclockReport`: `GET /api/v1/timeclock_report`.
  - Requires date range and location.
  - Supports deleted/clocked-in flags and user IDs.
- `ReportCardFiles`: `GET /api/v1/report_card_files`.
  - Supports number of days, limit, and location.

BI/source facts to preserve:

- Labor and timeclock rows.
- Active/deleted clock state.
- User IDs and location.
- Report-card file uploads, recency windows, and media/document activity.

Current gaps:

- There are no typed timeclock or report-card DTOs.
- There are no source snapshots or mappings into staff, media, or customer
  communication concepts.

### Webhooks

Implemented in `integrations/gingr/src/webhook.rs` with fixtures under
`docs/integrations/gingr/fixtures/webhooks/`:

- Signature verification uses HMAC-SHA256 over
  `webhook_type + entity_id + entity_type`.
- `Envelope` parses raw JSON but redacts debug output before verification.
- `Verified` exposes event type, entity ID, entity type, and payload only after
  verification.
- Event types modeled:
  - check-in, check-out, checking-in, checking-out
  - email sent
  - owner created/edited
  - animal created/edited
  - incident created/edited
  - lead created
  - unknown provider event
- Entity types modeled:
  - reservation
  - owner
  - animal
  - incident
  - lead
  - unknown provider entity
- Ack semantics match Gingr's retry contract:
  - 200 processed/no retry
  - 403 rejected/no retry
  - other status retryable

BI/source facts to preserve:

- Verified event type, entity type, and entity ID.
- Provider URL and received-at timestamp.
- Quarantined raw entity payload hash/body.
- Signature verification metadata.
- Ack decision.

Current gaps:

- Webhooks are verified but not projected into source snapshots.
- They do not yet drive incremental extraction jobs, data-quality queues, or BI
  update triggers.
- Public docs do not provide replay/backfill mechanics.
- Webhook-only facts must be reconciled with periodic endpoint pulls.

## Domain/app/storage surfaces relevant to BI

### Domain

Relevant current domain facts:

- `domain::entities::Reservation` includes reservation ID, location ID,
  customer ID, pet IDs, service, status, start/end timestamps, optional
  deposit, source, add-ons, and hard stops.
- `domain::entities::reservation::Status` includes inquiry, requested, missing
  info, vaccine pending, special review, waitlisted, offered, confirmed,
  checked in, active, checked out, cancelled, and rejected states.
- `domain::entities::Customer` includes name, contact, preferred contact, and
  portal account.
- `domain::entities::Pet` includes owner, name, species, birthday, sex,
  spay/neuter, temperament, and care profile.
- `domain::operations::TechnologyEcosystem` models core portal, data-access
  patterns, and adjacent systems.
- `domain::operations::DataAccessPattern` includes API, webhook, data export,
  warehouse, BI dashboard, and unknown.
- `domain::operations::AdjacentSystem` includes business intelligence and data
  lake.
- `domain::operations::DataQualityIssue` includes broad operational issues such
  as missing pet vaccination records, incomplete pet profiles, duplicate
  customers, open invoices, unclosed reservations, and inconsistent service
  naming.

Important distinction: `domain::operations::DataQualityIssue` is discovery
vocabulary. It is not yet a first-class source-data quality issue record with
provenance, severity, affected provider record, resolution status, or BI
visibility.

### App

Relevant current app facts:

- `app::tools::portal::Lookup` can look up provider records by provider,
  account, and criteria.
- Lookup outcomes can be customer, pet, reservation, not found, or ambiguous.
- `app::tools::portal::Provider` includes Gingr.
- `app::tools::portal::Include` includes customer contact, pet profile, and
  reservation ledger.
- Payment, messaging, document, media, and Hermes task/schedule ports exist.
- No app port yet models a BI extraction/projection workflow.

### Storage

Relevant current storage facts:

- `storage::operations::TechnologyEcosystemRecord` stores core portal, data
  access patterns, and adjacent systems.
- Storage codes mirror Gingr, API, webhook, data export, warehouse, BI
  dashboard, business intelligence, and data lake.
- There are no storage records yet for Gingr raw snapshots, extraction batches,
  provider record provenance, source hashes, data-quality issues, or BI
  fact/dimension outputs.

## Existing mapping coverage

Owner record:

- Current source shape: `response::OwnerRecord`.
- Current mapper: `mapping::customer::contact_candidate`.
- Promoted concept: customer contact candidate with name, email, mobile phone.
- Preserved raw facts: provider owner ID and unknown owner fields.

Animal record:

- Current source shape: `response::AnimalRecord`.
- Current mapper: `mapping::pet::name_candidate`.
- Promoted concept: pet name candidate.
- Preserved raw facts: provider animal ID, optional owner ID, species, birthday,
  and unknown animal fields.

Retail item:

- Current source shape: `dto::retail::Item`.
- Current mapper: `mapping::retail::product_candidate`.
- Promoted concept: retail product candidate with name, SKU/category, and active
  status.
- Preserved raw facts: provider item ID, quantity on hand, and unknown fields.

Webhook envelope:

- Current source shape: `webhook::Envelope` / `webhook::Verified`.
- Current mapper: verification gate only.
- Promoted concept: verified provider event boundary.
- Preserved raw facts: verified event/entity metadata and quarantined payload.

Reservation record:

- Current source shape: `response::ReservationRecord`.
- Current mapper: none.
- Promoted concept: none yet.
- Preserved raw facts: provider reservation ID, owner ID, animal ID, status, and
  unknown fields.

Reference record:

- Current source shape: `response::ReferenceRecord`.
- Current mapper: none.
- Promoted concept: none yet.
- Preserved raw facts: provider reference ID, name, and unknown fields.

## Provider gaps and risks for BI

- No first-class `gingr::snapshot` module exists yet.
- Endpoint DTOs and responses do not carry extraction batch ID, pulled-at
  timestamp, provider schema version, source URL/path, request parameters, or
  payload hash.
- No analytics/read-model crate or module exists yet.
- No explicit source-data quality issue record exists.
- Mapping errors are useful promotion errors, not durable BI-visible issue
  records.
- Reservation/stay is the highest-value BI slice but has thin mapping coverage.
- Owner, animal, and custom-field surfaces can carry high-sensitivity PII.
- Commerce and transaction surfaces are payment-sensitive.
- Date and location semantics are provider-specific.
- Public docs do not fully specify response schemas, pagination defaults, rate
  limits, error envelopes, deleted/merged records, or backfills.
- Grooming and training do not have documented service-specific DTOs.
- The repo correctly records grooming/training as provider gaps instead of
  inventing fake DTOs.

## Recommended source facts for the first BI snapshot lane

Start with reservation/stay snapshots because this joins the most downstream
needs:

- `source_system`: Gingr.
- `endpoint_name`: for example `reservations`, `reservations_by_owner`,
  `back_of_house`, `reservation_widget_data`, or `webhook:check_out`.
- `extraction_batch_id` and `pulled_at` or `received_at`.
- `request_scope`: app/subdomain/account label, location ID, date range,
  checked-in flag, API-user location caveat, and pagination/cursor.
- `provider_record_id`: reservation ID plus owner, animal, location, and service
  type IDs when present.
- `source_payload_hash` and quarantined raw payload reference.
- Provider status, lifecycle timestamps, service/reservation type,
  check-in/out facts, cancellation/completion flags, invoice/payment/deposit
  references, and add-ons.
- Data-quality issues such as missing owner, missing animal, missing location,
  missing time, unknown provider status, conflicting timestamps, unmapped
  service type, payment state conflict, unclosed reservation, duplicate provider
  record, or location-scope ambiguity.

This should feed a deterministic read-model projection such as
`analytics::stay::Fact` or a smaller `app::read_model::stay::Fact`. Raw Gingr
DTOs should not become the BI model.

## Questions for Tyler / BI team

1. What database engine or warehouse currently stores Gingr-derived BI data?
2. Is the current database raw payload storage, normalized operational tables,
   BI fact/dimension tables, or a mixture?
3. Which Gingr endpoints, reports, exports, or webhooks feed the current BI
   code?
4. Can we inspect the messy code path that pulls Gingr data and constructs the
   existing BI-facing types?
5. Which BI outputs are business-critical first: stay/utilization, revenue,
   capacity, retention, labor/revenue, service-line performance, or data-quality
   dashboards?
6. Which provider IDs are stable enough to join across endpoints, exports,
   webhooks, invoices, payments, and historical backfills?
7. How do they handle deleted, merged, duplicated, or re-created records?
8. Which statuses, service names, reservation types, invoice states, or custom
   fields have overloaded or location-specific meanings?
9. What fields are considered reliable versus known messy fields?
10. How far back does historical data need to be backfilled?
11. What cadence is required for incremental pulls?
12. How is multi-location access handled for API keys?
13. Are there non-API exports or reports that BI relies on because public API
    response schemas are incomplete?
14. What redaction/access policy is expected for raw owner, animal,
    custom-field, and payment payloads in snapshot storage?
15. Are webhook events used today for BI freshness, or only periodic pulls and
    exports?
16. Are grooming/training service facts encoded in reservation types, add-on
    services, custom fields, or a separate system?

## Next implementation implication

The next code task should not broaden endpoint builders first. It should add a
narrow source snapshot/provenance/data-quality contract around reservations and
stays, backed by fixture-only tests. That keeps Gingr provider vocabulary
quarantined, preserves messy source facts for audit/BI, and creates a semantic
path from raw provider pulls to domain validation and read-model projection.
