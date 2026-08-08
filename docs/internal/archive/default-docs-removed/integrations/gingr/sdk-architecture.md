# Gingr Rust SDK architecture

Status: architecture proposal for the future Rust SDK/adaptation layer. No SDK code, live Gingr calls, tenant domains, real customer data, or real API keys are introduced by this document.

Primary inputs:
- `docs/plans/2026-06-10-gingr-sdk-kanban-plan.md`
- `docs/integrations/gingr/README.md`
- `docs/integrations/gingr/sdk-endpoint-catalog.md`
- `docs/integrations/gingr/sdk-endpoint-catalog.json`
- `docs/integrations/gingr/sdk-webhooks.md`
- `docs/integrations/gingr/sdk-customer-portal-js.md`
- local article corpus under `docs/integrations/gingr/articles/`

## Executive decision

Create a dedicated Rust crate at `integrations/gingr`, with package name `gingr`, that models Gingr as an external provider boundary. The crate should expose typed read-only request builders, a redacted transport, quarantined provider DTOs, HMAC-verified webhook parsing, and explicit mapping adapters into the existing `domain` crate. It should not expose customer-facing portal JavaScript behavior or side-effecting Gingr operations as agent-executable commands.

Recommended workspace shape:

```text
nva/
  Cargo.toml
  domain/
  storage/
  apps/cli/
  integrations/
    gingr/
      Cargo.toml
      src/
        lib.rs
        config.rs
        transport.rs
        endpoint/
          mod.rs
          reservations.rs
          owners_animals.rs
          reference_data.rs
          commerce_retail.rs
          labor_ops.rs
          report_cards_files.rs
        response.rs
        webhook.rs
        mapping.rs
        fixtures.rs
      tests/
        config_redaction.rs
        endpoint_contracts.rs
        webhook_verification.rs
        mapping_boundaries.rs
```

Why `integrations/gingr` instead of `crates/gingr` or embedding in `domain`:
- Gingr is a provider boundary, not the pet-resort domain itself.
- The current workspace already has top-level semantic crates (`domain`, `storage`, `apps/cli`) rather than a `crates/` directory.
- A package named `gingr` keeps call sites semantic without repeating the workspace/product name.
- The `domain` crate remains the source of business truth; `gingr` translates provider wire data into domain candidates but does not own operational policy.

## Workspace and Cargo changes

Root `Cargo.toml`:

```toml
[workspace]
members = [
    "apps/cli",
    "domain",
    "storage",
    "integrations/gingr",
]
resolver = "3"

[workspace.dependencies]
# existing dependencies remain
async-trait = "0.1"
bon = "3.9"
chrono = { version = "0.4", features = ["serde"] }
domain = { path = "domain" }
nutype = { version = "0.7", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"

# new SDK dependencies
bytes = "1"
hex = "0.4"
hmac = "0.12"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
secrecy = "0.8"
serde_urlencoded = "0.7"
sha2 = "0.10"
subtle = "2"
url = "2"
wiremock = "0.6"
```

`integrations/gingr/Cargo.toml`:

```toml
[package]
name = "gingr"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
async-trait.workspace = true
bon.workspace = true
bytes.workspace = true
chrono.workspace = true
domain.workspace = true
hex.workspace = true
hmac.workspace = true
nutype.workspace = true
reqwest.workspace = true
secrecy.workspace = true
serde.workspace = true
serde_json.workspace = true
serde_urlencoded.workspace = true
sha2.workspace = true
subtle.workspace = true
thiserror.workspace = true
url.workspace = true

[dev-dependencies]
wiremock.workspace = true
```

Dependency notes:
- `reqwest` should be the default HTTP implementation, but endpoint code should depend on the SDK transport trait so tests and future CLIs can run without network access.
- `secrecy` should wrap API keys and webhook signature keys. If `secrecy` version policy changes later, preserve the semantic behavior: no `Debug`/`Display` of secret values and explicit exposure only at the transport/signature boundary.
- `subtle` should be used for constant-time signature comparison. If using a verified HMAC crate API that already provides constant-time verification, keep a test proving mismatches do not fall back to ordinary string equality logic.
- `wiremock` is a dev dependency only; the SDK tests must not touch live Gingr tenants.

## Public crate surface

`integrations/gingr/src/lib.rs` should make the semantic module map visible:

```rust
pub mod config;
pub mod endpoint;
pub mod fixtures;
pub mod mapping;
pub mod response;
pub mod transport;
pub mod webhook;

pub use config::{BaseUrl, ClientConfig, Provider, Subdomain};
pub use transport::Client;
```

Do not create a broad `prelude` until real call sites demonstrate which symbols should be ergonomic. Prefer module-qualified use at behavior call sites:

```rust
use gingr::{config, endpoint};

let request = endpoint::reference_data::GetLocations::builder()
    .build();
let locations = gingr_client.send(request).await?;
```

## Module map and responsibilities

### `config`

Own provider identity, base URL, tenant/subdomain validation, location identifiers, and secret wrappers.

Candidate types:
- `config::Subdomain`: validated tenant/app slug used in `https://{your_app}.gingrapp.com`.
- `config::BaseUrl`: parsed HTTPS base URL. Reject non-HTTPS and non-Gingr host shapes unless an explicit test-only constructor is used.
- `config::ApiKey`: secret wrapper for the user-account API `key` parameter.
- `config::WebhookSignatureKey`: secret wrapper for webhook HMAC verification.
- `config::LocationId`: provider-side location ID, distinct from `domain::location` IDs until mapping proves identity.
- `config::Provider`: provider identity value for audit/mapping (`Gingr` plus optional app label), not a global singleton.
- `config::ClientConfig`: builder requiring base URL and API key before any endpoint request can be sent.

Semantic constraints:
- API keys are supplied at process boundary and never persisted in fixtures or docs.
- `Debug`/`Display` for config must redact secrets and signed query/body fields.
- Location scope is request semantics, not a cosmetic filter; keep `config::LocationId` distinct from broader domain location identity.

### `transport`

Own HTTP mechanics, request encoding, response body capture, and redacted diagnostics.

Candidate types:
- `transport::Client<T = HttpTransport>`: configured SDK client.
- `transport::Transport` trait: async send of an already-built `endpoint::Request` into `response::Raw`.
- `transport::HttpTransport`: `reqwest` implementation.
- `transport::RequestParts`: method, path, encoded query/form fields, redaction metadata.
- `transport::RedactedRequest`: safe diagnostic representation.
- `transport::Error` and `transport::Result<T>`.

Rules:
- GET endpoints encode parameters as query strings.
- POST read endpoints encode parameters as `application/x-www-form-urlencoded` request bodies unless an endpoint contract later proves JSON.
- The API key is inserted by the client/transport, not by every endpoint builder.
- Redaction must apply to query strings, form bodies, request errors, tracing, and test failure output.
- Transport should preserve raw response bytes/status for boundary debugging, but raw bodies should remain in `response` quarantine before mapping.

### `endpoint`

Own typed read/query request builders and documented endpoint invariants. This module should not own business policy or provider response interpretation beyond endpoint-specific DTO parsing.

Shape:

```text
endpoint::Request trait
endpoint::Method enum
endpoint::Date
endpoint::DateRange
endpoint::PageWindow
endpoint::reservations::*
endpoint::owners_animals::*
endpoint::reference_data::*
endpoint::commerce_retail::*
endpoint::labor_ops::*
endpoint::report_cards_files::*
```

Endpoint request objects should:
- encode their method/path and typed parameters;
- expose redaction metadata for sensitive fields;
- enforce date/range/page invariants before transport;
- use `bon` builders where construction has meaningful optional fields;
- use enums/newtypes for provider values such as `RestrictTo`, invoice kind/open-closed filters, reservation type IDs, owner/animal/reservation IDs, and date formats;
- not return raw `serde_json::Value` to agent-facing code.

### `response`

Own provider wire DTOs and raw payload quarantine.

Candidate types:
- `response::Raw`: HTTP status, headers if needed, and body bytes/string. Boundary-only.
- `response::Envelope<T>`: documented Gingr JSON envelope where known (`success`, `error`, `data`) without forcing all endpoints into a false universal shape.
- `response::ProviderPayload`: explicit raw JSON value for not-yet-modeled fields.
- Endpoint-specific DTO modules or types, for example `response::locations::Location`, when the source corpus or fixtures prove fields.
- `response::Error` and `response::Result<T>`.

Rules:
- Unknown fields are preserved during early implementation, preferably with `#[serde(flatten)]`, but mapped domain decisions must ignore unknown/raw fields unless a typed mapper explicitly promotes them.
- PII and payment-sensitive DTOs (`custom_field_search`, owner/animal records, transaction/payment details, email recipients, notes/HTML) must not be logged raw.
- Provider booleans, money, IDs, nullable/empty-string fields, and timestamps should remain provider DTOs until `mapping` validates and names their domain meaning.

### `webhook`

Own inbound Gingr webhook parsing, signature verification, typestate gate, event/entity enums, and receiver acknowledgement semantics.

Candidate types:
- `webhook::EventType`: `CheckIn`, `CheckOut`, `CheckingIn`, `CheckingOut`, `EmailSent`, `OwnerCreated`, `OwnerEdited`, `AnimalCreated`, `AnimalEdited`, `IncidentCreated`, `IncidentEdited`, `LeadCreated`, `Unknown(String)`.
- `webhook::EntityType`: `Reservation`, `Owner`, `Animal`, `Incident`, `Lead`, `Unknown(String)`.
- `webhook::EntityId`: normalized string form used in the signature input.
- `webhook::Signature`: validated lowercase hex HMAC-SHA256 digest.
- `webhook::UnverifiedEnvelope`: parsed JSON and verification inputs only.
- `webhook::VerifiedEnvelope`: only constructible via `UnverifiedEnvelope::verify(&config::WebhookSignatureKey)`.
- `webhook::Ack`: `Processed`, `RejectedPermanently`, `RetryableFailure`.
- `webhook::VerificationError` and `webhook::Result<T>`.

Verification rule from the corpus:
- HMAC-SHA256 key: configured webhook signature key.
- Message: `webhook_type` + `entity_id` + `entity_type`, concatenated with no separators.
- Output: hex-encoded SHA256 HMAC digest.

Typestate rule:

```text
Raw body
  -> webhook::UnverifiedEnvelope
  -> verify(signature_key)
  -> webhook::VerifiedEnvelope
  -> mapping::webhook_event(...)
  -> domain workflow candidate
```

`entity_data`, `email_data`, and `recipients` must not be exposed to domain or agent workflows from `UnverifiedEnvelope`.

### `mapping`

Own explicit conversions from Gingr DTOs and verified webhook envelopes into existing `domain` contracts. This module should be intentionally conservative.

Candidate structure:

```text
mapping::reservation
mapping::customer
mapping::pet
mapping::location
mapping::money
mapping::webhook
mapping::error
```

Rules:
- Mapping converts provider IDs to provider-scoped references or domain IDs only when identity semantics are proven.
- Raw provider payloads never drive agent decisions directly.
- Mapping errors are semantic and local: `mapping::Error::MissingRequiredProviderField`, `mapping::Error::UnsupportedProviderStatus`, `mapping::Error::AmbiguousIdentity`, etc.
- Agent workflows should receive domain values or explicit review candidates, not `gingr::response::*` DTOs.
- Unsupported statuses/fields should produce reviewable gaps rather than silent default values.

Examples of safe mapping targets from the current domain crate:
- `domain::location` concepts for location references after provider identity is established.
- `domain::reservation` concepts for availability/reservation status candidates after Gingr status semantics are modeled.
- `domain::customer::{Name, Email, Phone}` and `domain::pet::Name` for validated contact/pet fields.
- `domain::workflow` or `domain::tools` candidate packets for agent recommendations, never automatic writes.

### `fixtures`

Own compile-time or test-only access to sanitized samples.

Sources:
- `docs/integrations/gingr/fixtures/webhooks/reservation-check-out.json`
- `docs/integrations/gingr/fixtures/webhooks/email-sent.json`
- future sanitized endpoint fixtures derived from docs or approved test tenants.

Rules:
- Fixtures contain fake keys, synthetic/sanitized names, fake emails, fake phone numbers, fake tenant domains, and no production customer data.
- Fixture helpers may expose bytes/strings for tests; they should not become runtime defaults.
- Fixture signatures should be recomputable from a fake key such as `test-webhook-signature-key`.

## Semantic type boundaries

Use explicit types at every boundary where Gingr's wire model can lie or drift:

Provider identity and auth:
- `config::Subdomain`, `config::BaseUrl`, `config::ApiKey`, `config::Provider`.
- Do not pass base URLs or API keys as naked strings outside config construction.

Provider IDs:
- `config::LocationId` or `endpoint::location::Id` for Gingr location IDs.
- `endpoint::reservation::Id`, `endpoint::owner::Id`, `endpoint::animal::Id`, `endpoint::service::TypeId`, `endpoint::subscription::Id`, etc., as provider IDs.
- Do not alias provider IDs directly to `domain::*Id` unless reconciliation has proven they are the same identity space.

Dates and pagination:
- `endpoint::Date` for `YYYY-MM-DD` Gingr parameters.
- `endpoint::IsoDate` only for the endpoints whose nested params are documented as ISO 8601.
- `endpoint::DateRange` should enforce the `reservations` 30-day limit where applicable.
- `endpoint::invoice::PageWindow` should encode Gingr's unusual `page` as starting-result value, not a generic page number.
- `endpoint::subscription::LimitOffset` should encode `limit` and `offset` together.

Status/filter enums:
- `endpoint::reservation::RestrictTo`: `PendingRequests`, `CurrentlyCheckedIn`, `Future`, `Past`, `WaitListed`.
- Invoice booleans such as `complete` and `closed_only` should become meaningful enums or small value objects rather than raw booleans at call sites.
- Webhook event/entity names become `webhook::EventType` and `webhook::EntityType`.

Provider payload quarantine:
- DTOs may contain raw strings/booleans/nulls/empty strings because Gingr examples show those shapes.
- Domain and agent modules must consume mapped semantic values or explicit review candidates.
- Unknown provider fields may be preserved for diagnostics but cannot silently alter policy.

Errors:
- Each meaningful module should define a local `Error` and `Result<T>`.
- Avoid `anyhow`, `Box<dyn Error>`, or string errors in the SDK library surface. CLI/application layers may erase errors for reporting.

## Secret redaction strategy

Secrets and sensitive payloads must be redacted by construction, not by operator discipline.

Required strategy:
1. Wrap API keys and webhook signature keys in secret newtypes (`config::ApiKey`, `config::WebhookSignatureKey`) backed by `secrecy::SecretString` or equivalent.
2. Implement `Debug`/`Display` for public config/request diagnostic types so secret values render only as `<redacted>`.
3. Keep endpoint builders unaware of the actual API key; transport adds `key` immediately before sending.
4. Represent request parameters as typed fields plus redaction metadata. Never build an unredacted URL string and then try to scrub it later.
5. Provide `transport::RedactedRequest` and assert in tests that query strings, form bodies, errors, and tracing-style output do not include sentinel secrets.
6. Treat high-PII/payment response bodies as sensitive even when they are not credentials: owner/animal records, custom field search, email recipients, notes/HTML, transaction details, payments, portal event details.
7. Fixture policy: fake domains, fake keys, fake emails/phones/names only; no real API key-like tokens.

Test sentinel values:
- API key: `gingr_test_api_key_do_not_send`
- Webhook signature key: `test-webhook-signature-key`
- Tenant/app: `example-pet-resort`

A redaction test should fail if any sentinel secret appears in `Debug`, `Display`, URL, body diagnostic, error message, fixture report, or snapshot.

## Endpoint release plan: v0 vs v1

The source catalog contains 33 included read/query endpoints and 2 explicitly excluded side-effect endpoints. The release split below is exact for the SDK planning surface.

### v0: safe operational read foundation

Implement these 17 endpoints first:

`reference_data`:
- `get_locations`
- `get_species`
- `get_breeds`
- `get_vets`
- `get_temperaments`
- `get_immunization_types`
- `get_animal_immunizations`

`reservations`:
- `reservation_types`
- `get_services_by_type`
- `reservation_widget_data`
- `reservations`
- `reservations_by_animal`
- `reservations_by_owner`
- `back_of_house`

`owners_animals`:
- `owner`
- `owners`
- `animals`

Rationale:
- This covers provider configuration/reference data and the core reservation/owner/animal read model needed for safe AI operational intelligence.
- It includes the plan's first safe slice (`get_locations`, `reservation_types`, `reservation_widget_data`, `owners`, `animals`) plus the closely related typed reservation and reference-data endpoints needed to make mappings meaningful.
- It avoids credential-verification, SQL-like custom field searches, payment/transaction details, labor records, report-card file access, and side-effect endpoints in the first implementation wave.

### v1: expanded read coverage with stronger sensitivity gates

Implement these 16 endpoints after v0 redaction, transport, webhook, and mapping gates are proven:

`reservations`:
- `existing_reservation_estimate`
- `recently_cancelled_reservations`

`owners_animals`:
- `new_modified_owners`
- `forms_get_form`
- `custom_field_search`
- `get_feeding_info`
- `get_medication_info`
- `authorize_owner`

`commerce_retail`:
- `get_all_retail_items`
- `list_transactions`
- `transaction`
- `list_invoices`
- `get_subscription`
- `get_subscriptions`

`labor_ops`:
- `timeclock_report`

`report_cards_files`:
- `report_card_files`

Rationale:
- These are still cataloged as read/query surfaces, but several have higher sensitivity or less certain schemas: credential verification (`authorize_owner`), arbitrary custom-field/SQL-like filters (`custom_field_search`), payment/transaction/invoice/subscription details, labor/timeclock records, report-card files, feeding/medication details, and recently cancelled reservation context.
- They should wait until v0 proves redaction, response quarantine, fixtures, and mapping review gaps.

### Excluded from v0/v1 read-only SDK surface

Do not implement these as normal SDK read endpoints:
- `quick_checkin`: side effect; checks in pets, creates a reservation if one does not exist, and checks it in.
- `receive_call`: side effect; triggers in-app alert and records an incoming phone call.

If a later business need requires either endpoint, design a separate human-reviewed side-effect command surface with dry-run/review gates. Do not hide them under the read-only client.

## Webhooks release plan

Webhook support can be developed alongside endpoint v0 because it is a separate inbound boundary:

v0 webhook scope:
- parse `UnverifiedEnvelope`;
- verify HMAC signatures into `VerifiedEnvelope`;
- expose `EventType`, `EntityType`, `EntityId`, `Signature`, and `Ack`;
- test with sanitized fixtures under `docs/integrations/gingr/fixtures/webhooks/`;
- provide mapping stubs or conservative candidates for reservation check-in/check-out/checking-in/checking-out and email-sent events only where fixtures prove fields.

v1 webhook scope:
- add event/entity-specific DTOs and mappers for owner, animal, incident, and lead events when fixtures or live approved test captures prove shape;
- add durable inbox/receiver helpers only in an application crate, not in the core SDK, unless explicitly scoped.

## Customer Portal JavaScript boundary

Customer Portal JavaScript is not part of this Rust backend SDK crate.

Allowed future SDK-adjacent deliverables:
- docs/examples listing typed browser event names: `reservation_created`, `owner_created`, `lead_created`;
- minimal manually installed browser snippets for reviewed analytics use;
- redaction/minimization examples for portal event forwarding.

Not allowed in the core `gingr` crate:
- Admin writes that install JavaScript or CSS;
- DOM manipulation;
- event listener runtime behavior in `GingrClient`;
- automatic forwarding of raw `e.detail` into analytics, LLM prompts, agent memory, or commands.

## Tests and gates

Architecture/documentation gate for this card:
- this document exists at `docs/integrations/gingr/sdk-architecture.md`;
- it references the endpoint catalog, webhook catalog, customer-portal boundary, and current workspace layout;
- it names crate path, workspace/Cargo changes, module map, semantic boundaries, secret redaction, tests/gates, and v0/v1 endpoint split.

Implementation gates for the future SDK card:

Unit tests:
- config rejects invalid subdomains/base URLs and redacts API keys.
- `Debug`/`Display` for config, request, transport error, and redacted diagnostics never include sentinel secrets.
- endpoint builders enforce required params, date formats, `reservations` 30-day range, owner lookup one-of semantics, invoice page/per-page pairing, subscription limit/offset semantics, and location-scope types.
- excluded endpoints (`quick_checkin`, `receive_call`) are not exported by `endpoint` modules.
- webhook HMAC verification succeeds for doc-derived fake-key fixtures and fails for malformed/mismatched signatures.
- `UnverifiedEnvelope` cannot expose payload fields to mapping/domain APIs; only `VerifiedEnvelope` can be mapped.
- DTO parsers preserve unknown fields while mapping ignores them unless explicitly promoted.

Integration/mock tests:
- `wiremock` or mock `transport::Transport` verifies method/path/query/form encoding for v0 endpoints.
- POST read endpoints use form encoding and are still treated as read requests.
- HTTP errors preserve redacted diagnostics without dumping raw high-PII bodies.

Semantic/domain tests:
- mappers produce `domain` values or explicit mapping errors; they do not leak raw `serde_json::Value` or provider strings into agent decision packets.
- unsupported provider statuses and ambiguous IDs produce reviewable mapping failures.
- agent-facing packets separate observed provider facts, mapped domain candidates, and recommended actions requiring review.

Repository gates:
- `cargo fmt --all --check`
- `cargo check --workspace --all-features`
- `cargo test --workspace --all-features`
- optional doc/link check for `docs/integrations/gingr/*.md`

Safety gates:
- no live Gingr network calls in tests;
- no real tenant domains, API keys, customer records, payment details, or credentials in fixtures;
- no customer-facing actions, writes, check-ins, call notifications, payments, or portal JavaScript changes are introduced by the read SDK.

## Implementation order

1. Add `integrations/gingr` crate and workspace dependencies.
2. Add config secret/base-url types and redaction tests.
3. Add transport trait, mock transport, HTTP transport skeleton, and redacted request diagnostics.
4. Add v0 endpoint request builders and request-encoding tests.
5. Add response quarantine types and minimal DTOs for proven v0 response shapes.
6. Add webhook typestate/HMAC verifier and fixture tests.
7. Add conservative mapping adapters into `domain` for values whose semantics are proven.
8. Run workspace gates and write a readiness review before adding v1 sensitive endpoints.

## Open questions for the implementation card

These should not block the architecture document, but they should be answered during implementation:
- Should `config::BaseUrl` allow non-Gingr hosts behind a `test-transport` or `test-base-url` feature for local wiremock URLs, or should tests inject transport without ever constructing HTTP base URLs?
- Which existing `domain` IDs are provider-scoped versus internal canonical IDs? Until answered, map Gingr IDs to provider-scoped references rather than domain IDs.
- Should `authorize_owner` be omitted entirely from v1 unless a reviewed credential-verification use case appears? It is cataloged in v1 here only as an explicit sensitivity-gated endpoint, not as a normal ingestion primitive.
- How should arbitrary `owners`/`animals` `params[*]` filters be allowlisted to avoid exposing SQL-like provider expressions to agent code?
