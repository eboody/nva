# Gingr SDK readiness review and NVA AI-program brief

Review date: 2026-06-10

Scope: final integration review of the local Gingr SDK source corpus, documentation catalogs, and Rust `integrations/gingr` implementation slice. This review used only the repository-local corpus and tests. No live Gingr tenant, API key, webhook secret, production customer data, or member-facing workflow was exercised.

## Executive verdict

The Gingr SDK foundation is ready for a v0 internal/prototype integration slice: typed read-only request builders, redacted API-key handling, provider DTO quarantine, conservative mapping adapters, and HMAC-verified webhook parsing are present and covered by passing Rust gates.

It is not production-operational yet. Production use remains gated by approved Gingr tenant/sandbox access, real read-only response captures, rate-limit/error-envelope validation, multi-location behavior checks, and a reviewed receiver/application layer for webhook ingestion.

## Evidence reviewed

### Source corpus and catalog documents

- `docs/integrations/gingr/source-audit.md`
- `docs/integrations/gingr/README.md`
- `docs/integrations/gingr/sdk-endpoint-catalog.md`
- `docs/integrations/gingr/sdk-architecture.md`
- `docs/integrations/gingr/sdk-webhooks.md`
- `docs/integrations/gingr/sdk-customer-portal-js.md`
- `docs/integrations/gingr/articles/`
- `docs/integrations/gingr/fixtures/webhooks/`

### Rust SDK implementation surfaces

- `integrations/gingr/Cargo.toml`
- `integrations/gingr/src/lib.rs`
- `integrations/gingr/src/config.rs`
- `integrations/gingr/src/transport.rs`
- `integrations/gingr/src/endpoint/`
- `integrations/gingr/src/response.rs`
- `integrations/gingr/src/mapping/`
- `integrations/gingr/src/webhook.rs`
- `integrations/gingr/tests/config_redaction.rs`
- `integrations/gingr/tests/endpoint_contracts.rs`
- `integrations/gingr/tests/expanded_endpoint_contracts.rs`
- `integrations/gingr/tests/webhook_contracts.rs`

Note: the workspace directory is not a Git checkout in this run, so this review could not use `git diff` or commit metadata. It inspected files directly and reran the Rust gates from the workspace root.

## Checklist review

### 1. Source links and catalogs complete enough for v0

Status: pass for v0; known non-blocking gaps for production completeness.

The source audit records a 32-article local corpus plus one known stale 404. The v0-critical documents cite exact public Gingr links for:

- API overview/read-only posture: `26687020531853` and `27482358729101`.
- API keys, HTTPS JSON, and `Can Access API` permission: `25721874372109`.
- Endpoint request catalog: `25722122517517`.
- Webhook activation/signature key: `25660732990477`.
- Webhook events, HMAC rule, and retry/status semantics: `25660010986125`.
- Webhook payload examples: `25659829911565`.
- Customer Portal JavaScript emitted/listener events: `25846095500557` and `25846282748045`.

The endpoint catalog is complete enough for the documented v0 read foundation. The v0 slice in `sdk-architecture.md` names 17 safe operational read endpoints:

- `reference_data`: `get_locations`, `get_species`, `get_breeds`, `get_vets`, `get_temperaments`, `get_immunization_types`, `get_animal_immunizations`.
- `reservations`: `reservation_types`, `get_services_by_type`, `reservation_widget_data`, `reservations`, `reservations_by_animal`, `reservations_by_owner`, `back_of_house`.
- `owners_animals`: `owner`, `owners`, `animals`.

The Rust implementation exposes those v0 endpoints and additionally includes several expanded read/sensitive endpoint builders, with tests covering the expanded contracts. This is acceptable because the code keeps side-effect endpoints out and marks sensitive/high-PII surfaces as quarantine/review boundaries.

Production gaps remain explicit: public docs only partially define response schemas, error envelopes, rate limits, pagination defaults, sandbox setup, webhook replay behavior, and multi-location semantics. Those gaps require sandbox/live read-only probes before a production claim.

### 2. SDK preserves Gingr read-only boundary and API-key secrecy

Status: pass.

Read-only boundary:

- `sdk-endpoint-catalog.md` explicitly excludes `quick_checkin` and `receive_call` from the read-only SDK surface because they cause operational side effects.
- `integrations/gingr/src/endpoint/catalog.rs` exports only read/query endpoint names; `quick_checkin` and `receive_call` are not exported.
- `endpoint_contracts.rs` includes a guard test named `excluded_side_effect_endpoints_are_not_exported_as_request_builders`.
- POST read endpoints are modeled as form-encoded read/query calls rather than agent mutations.

API-key secrecy:

- `config::ApiKey` wraps `secrecy::SecretString` and exposes the value only through `expose_for_transport` inside the transport boundary.
- `transport::Client::capture_request` injects `key` centrally; endpoint builders do not own the API key.
- `transport::RequestParts::redacted` marks sensitive parameter names and displays `key=<redacted>`.
- `ClientConfig`, `ApiKey`, and redacted request display/debug paths are covered by sentinel-secret tests.
- No real API key or tenant credential appears in fixtures or docs reviewed.

Caveat: `transport::HttpTransport` intentionally returns `HttpNotImplemented` in this SDK slice. That keeps the crate safe for request-construction tests but means live API use still requires a future reviewed transport implementation and sandbox gate.

### 3. Webhooks require signature verification before domain/agent use

Status: pass.

`integrations/gingr/src/webhook.rs` enforces the expected boundary:

- Raw JSON parses into `WebhookEnvelope`.
- `WebhookEnvelope` exposes only verification metadata before verification: event type input, entity type input, and signature input.
- `WebhookEnvelope::verify(&WebhookSignatureKey)` computes HMAC-SHA256 over `webhook_type + entity_id + entity_type` with no separators.
- Entity IDs normalize from documented string or integer JSON forms before signature verification.
- Signature comparison uses `subtle::ConstantTimeEq` after lowercase hex decoding.
- Only `VerifiedWebhook` exposes `VerifiedWebhookPayload`, and only verified payloads expose `entity_data`, `email_data`, and `recipients`.
- `WebhookAck` models Gingr's `200`, `403`, and retryable status behavior.

Tests verify both positive fixtures and negative boundaries:

- `reservation_check_out_fixture_verifies_before_payload_can_be_inspected`
- `email_sent_fixture_normalizes_numeric_entity_id_for_signature_verification`
- `verification_rejects_tampered_signatures_without_exposing_secret_material`
- `verification_reports_unsupported_entity_id_and_malformed_signature_boundaries`
- `unverified_debug_output_exposes_only_verification_metadata_not_payload`
- `receiver_ack_semantics_match_gingr_retry_contract`

### 4. Rust tests/gates

Status: pass in this run.

Commands run from `/home/eran/code/pet-resort-agent-foundation`:

```text
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Result: all commands exited successfully.

Observed test coverage in `cargo test --workspace`: 108 tests passed across domain, storage, and Gingr integration tests; 0 failed. Several harnesses correctly reported 0 unit/doc tests where no tests are defined, while the relevant integration suites executed.

### 5. Semantic fidelity doctrine

Status: pass for this foundation slice.

The implementation preserves semantic boundaries rather than flattening provider data into domain decisions:

- Provider identity/auth/config values are typed: `Subdomain`, `BaseUrl`, `ApiKey`, `Provider`, `ClientConfig`.
- Request construction is domain/provider-specific: `endpoint::Date`, `IsoDate`, `DateRange`, `Limit`, provider ID newtypes, builder structs, and enums such as `RestrictTo`.
- Read/query request builders encode endpoint-specific invariants such as 30-day reservation ranges, one-of owner lookup discriminators, invoice pagination pairing, subscription limit/offset, location scoping, and POST form-body behavior.
- Provider wire responses remain under `response` and mapping surfaces. `ProviderPayload` and `serde_json::Value` are quarantined boundary values, not policy inputs.
- Mapping uses semantic candidates and local errors (`mapping::Error`) instead of allowing raw payloads to drive agent workflows.
- Webhook verification uses typestate-like separation: unverified envelope -> verified webhook -> mapping/domain candidate.
- High-sensitivity endpoint surfaces remain named and reviewable: custom-field search, transaction/payment details, timeclock/labor records, report-card files, feeding/medication information, and customer credential verification.

The current shape is intentionally conservative: it builds the provider boundary and request contracts before claiming rich business mapping. That is the right semantic posture until live/sandbox response schemas prove field meanings.

## NVA AI-program talking points

### What this SDK enables

- Safe operational read-model foundation for Gingr-backed locations: locations, species/breeds, vets, temperaments, immunization types/records, reservation types, reservation widget summaries, reservations by date/owner/animal, back-of-house check-in/check-out views, owners, and animals.
- A typed boundary between Gingr provider data and NVA/PetSuites domain concepts, preventing raw Gingr JSON from becoming hidden policy logic.
- Redacted, testable request construction that keeps API keys out of logs, docs, fixtures, debug output, and endpoint builders.
- Webhook receiver primitives that require HMAC verification before event payloads can be mapped into workflow candidates.
- Internal recommendation workflows for staffing, capacity, reservation triage, daycare/grooming/training/retail opportunities, and operational briefs, provided those workflows consume mapped domain candidates rather than raw provider payloads.
- Customer Portal JavaScript guidance as a separate browser/analytics observation boundary, not as backend SDK behavior.

### What remains gated by sandbox/API access

- Real response schemas for each endpoint, including field types, nullable/empty-string behavior, status strings, money formats, timestamps, and unknown fields.
- Gingr error envelopes and HTTP status behavior for failed read requests.
- Rate limits, pagination defaults, maximum page sizes, and large-result behavior.
- Multi-location semantics, especially endpoints affected by the API user's currently selected/location-scoped context.
- Validation that `HttpTransport` can send redacted HTTPS requests without leaking secrets in URL/body/error/tracing paths.
- Webhook replay/delivery behavior against a real or sandbox receiver, including retry timing and signature edge cases.
- Production identity reconciliation between Gingr provider IDs and NVA/PetSuites domain/customer/pet/location identities.
- Privacy review for high-PII and payment-sensitive endpoints before any model prompt, analytics export, or downstream storage use.

### Draft/recommendation-only workflows

These can be implemented as internal drafts or staff-review candidates after sandbox read validation, but should not execute member-facing actions automatically:

- Daily operational brief: arrivals/departures, overnights, daycare groups, grooming/training readiness, high-attention pets, and staffing pressure.
- Reservation triage: missing immunizations, medication/feeding flags, temperament or behavior-review needs, and capacity conflicts.
- Retail, package, grooming, training, or daycare upsell candidates generated from mapped service history and eligibility constraints.
- Lead, owner, or portal-conversion analytics from documented portal/browser events, with minimized/redacted payloads.
- Webhook-derived event inbox items such as checked-in/checked-out candidates, email-sent observations, or owner/animal change review tasks.

### Human-reviewed execution only

These require explicit human review, and likely additional privacy/compliance and customer-facing QA, before execution:

- Any customer/member-facing message, reminder, offer, legal/payment/immunization request, or follow-up generated from Gingr data.
- Any action that writes to Gingr, changes reservation/check-in state, creates records, or affects the customer portal.
- Use of side-effect endpoints such as `quick_checkin` or `receive_call`.
- Credential-verification flows using `authorize_owner` or any workflow that handles pet-parent passwords.
- Payment/transaction/invoice handling beyond internal review summaries.
- Forwarding raw owner, lead, pet, medical, medication, feeding, notes, email body, or report-card payloads into model prompts, analytics systems, agent memory, or external tools.
- Installing Customer Portal JavaScript/CSS or changing browser/customer-facing behavior.

## Final readiness position

The board's Gingr SDK outputs meet the v0 readiness bar for source-backed, read-only, secret-redacted, semantically typed SDK foundations. The next safe milestone is not production rollout; it is a narrow sandbox validation card that implements/reviews `HttpTransport`, captures sanitized read-only fixtures for the v0 endpoint set, confirms webhook receiver behavior with fake/sandbox events, and tightens mappings only where provider fields are proven.
