# Gingr SDK Kanban Plan

> **For Hermes:** Use the `pet-resort-gingr-sdk` Kanban board to execute this plan. Keep code-mutating SDK work gated until the current `pet-resort-service-domain-contracts` code board is no longer mutating `/home/eran/code/pet-resort-agent-foundation`.

**Goal:** Build a Rust SDK/adaptation layer for Gingr so pet-resort AI agents can read operational data, verify webhooks, and translate raw Gingr payloads into semantic domain contracts without letting raw strings/booleans leak into agent decisions.

**Architecture:** Treat Gingr as a boundary provider. The SDK should expose typed request builders, a configurable transport, endpoint-specific response envelopes, webhook verification/parsing, and semantic mappers into the existing `domain` contracts. Gingr’s public API is documented as HTTPS JSON and read-only; any agent action that affects customers, payments, operations, or records must remain a draft/recommendation or explicit human-reviewed command outside this SDK.

**Tech Stack:** Rust 2024 workspace, current crates `domain`, `storage`, `apps/cli`; likely new crate `integrations/gingr` or `gingr`; dependencies to evaluate: `reqwest`, `url`, `hmac`, `sha2`, `hex`, `secrecy`, `serde_urlencoded`, `wiremock`/mock transport.

---

## Source inventory

Canonical local inventory:
- `docs/integrations/gingr/README.md`
- `docs/integrations/gingr/manifest.json`
- `docs/integrations/gingr/articles/`

Canonical public source category:
- https://support.gingrapp.com/hc/en-us/categories/25361341525645-Developer-Guide

Key source articles:
- API overview/read-only notice: https://support.gingrapp.com/hc/en-us/articles/26687020531853-API-Feature-Overview
- API keys/read-only HTTPS JSON notice: https://support.gingrapp.com/hc/en-us/articles/25721874372109-Access-API-Keys-How-To
- API functions reference: https://support.gingrapp.com/hc/en-us/articles/25722122517517-Gingr-API-Functions-Reference
- Webhooks topic: https://support.gingrapp.com/hc/en-us/articles/26688281811597-Webhooks-Topic-Outline
- Activate webhooks: discovered in `docs/integrations/gingr/manifest.json`
- Webhook event types/response codes/HMAC signature: https://support.gingrapp.com/hc/en-us/articles/25660010986125-Event-Types-and-Response-Codes-Reference
- Webhook payload examples: https://support.gingrapp.com/hc/en-us/articles/25659829911565-Event-Data-Structure-Examples-Reference
- Custom CSS/JavaScript topic: https://support.gingrapp.com/hc/en-us/articles/26687431397901-Custom-CSS-JavaScript-Topic-Outline
- JavaScript events emitted: https://support.gingrapp.com/hc/en-us/articles/25846095500557
- Listen for JS events: https://support.gingrapp.com/hc/en-us/articles/25846282748045-Listen-for-Custom-JavaScript-Events-Reference
- Integrations overview/Zapier: https://support.gingrapp.com/hc/en-us/articles/26689830918669-Integrations-Topic-Outline

Known caveat:
- One related article URL, `https://support.gingrapp.com/hc/en-us/articles/26686883788685`, returned 404 during crawl; keep it as a stale-source note, not an implementation blocker.

---

## SDK pieces to identify and implement

### 1. Source normalization and endpoint catalog

Deliverables:
- `docs/integrations/gingr/sdk-endpoint-catalog.md`
- machine-readable endpoint table candidate: `docs/integrations/gingr/sdk-endpoint-catalog.json`

Must extract:
- base URL pattern: `https://{your_app}.gingrapp.com`
- auth shape: per-user `key` parameter; user must have “Can Access API”; never log/print API keys
- read-only boundary from docs
- endpoint method/path, purpose, required params, optional params, date/range constraints, pagination rules, location scoping caveats
- grouping by SDK module:
  - reservations: `reservations`, `reservation_widget_data`, `reservations_by_animal`, `reservations_by_owner`, `recently_cancelled_reservations`, `reservation_types`, `get_services_by_type`, `existing_reservation_estimate`, `back_of_house`
  - owners/animals: `owner`, `owners`, `animals`, `new_modified_owners`, `authorize_owner`, `forms/get_form`, `custom_field_search`
  - reference data: `get_locations`, `get_species`, `get_breeds`, `get_vets`, `get_temperaments`, `get_immunization_types`, `get_animal_immunizations`
  - commerce/retail: `get_all_retail_items`, `list_transactions`, `transaction`, `list_invoices`, `get_subscription`, `get_subscriptions`
  - labor/ops: `timeclock_report`
  - report cards/files: `report_card_files`

### 2. Webhook contract catalog

Deliverables:
- `docs/integrations/gingr/sdk-webhooks.md`
- fixture samples under `docs/integrations/gingr/fixtures/webhooks/` if useful

Must extract:
- event enum: `check_in`, `check_out`, `checking_in`, `checking_out`, `email_sent`, `owner_created`, `owner_edited`, `animal_created`, `animal_edited`, `incident_created`, `incident_edited`, `lead_created`
- HMAC rule: SHA256 HMAC of concatenated `webhook_type`, `entity_id`, `entity_type`, keyed by configured signature key
- response semantics: 200 means processed/no retry, 403 means intentionally reject/no retry, any other code retries up to 10 times with increasing timeouts
- payload envelopes: common fields `webhook_url`, `webhook_type`, `entity_id`, `entity_type`, `signature`, `entity_data`; email events include `email_data` and `recipients`
- safety rule: signature verification must be an explicit typestate/gate before payload can enter agent/domain workflow

### 3. Customer Portal JS/event integration boundary

Deliverables:
- `docs/integrations/gingr/sdk-customer-portal-js.md`

Must extract:
- custom CSS/JS capability is customer-facing portal customization, not backend API
- JavaScript events emitted/listenable by portal integrations
- safe use-cases for AI program: analytics capture, conversion tracking, lead intent signals, portal UX observations
- unsafe use-cases requiring explicit review: silently modifying customer-facing behavior, collecting more personal data than intended, auto-submitting portal actions

### 4. Rust crate/module architecture

Deliverables:
- `docs/integrations/gingr/sdk-architecture.md`
- code target proposal in the doc: crate path, module names, dependency additions, test strategy

Recommended module shape:
- `config`: `GingrSubdomain`, `GingrBaseUrl`, `GingrApiKey`/secret wrapper, `LocationId`, provider identity
- `transport`: trait `GingrTransport`, default HTTP client, form/query encoding, redacted request logging
- `endpoint`: typed endpoint marker/request builder per API function, date-range validation, pagination helpers
- `response`: common Gingr envelope, raw payload quarantine, endpoint-specific DTOs
- `webhook`: `WebhookEnvelope`, `WebhookEventType`, `EntityType`, `VerifiedWebhook`, `WebhookSignatureKey`, verification errors
- `mapping`: conversions from DTO/webhook data to existing `domain` types; never let raw provider payload drive policies directly
- `fixtures`: sanitized samples from docs

### 5. Core SDK implementation: config + transport + endpoint scaffolding

Deliverables:
- crate added to workspace (exact path decided by architecture card)
- typed config/base URL/API-key handling
- transport abstraction and mock transport tests
- endpoint request builders for first safe slice:
  - `get_locations`
  - `reservation_types`
  - `reservation_widget_data`
  - `owners`
  - `animals`
- tests prove API key redaction, URL/path construction, query/form encoding, date validation, and no write endpoints are exposed as mutating commands

### 6. Reservation/owner/animal/reference endpoint expansion

Deliverables:
- typed request/response DTOs for operational read models
- tests for location scoping, date ranges, restrictions, pagination/filter params
- semantic conversion adapters where existing domain types are ready; documented intentional gaps otherwise

### 7. Commerce/retail/labor/report-card endpoint expansion

Deliverables:
- typed request/response DTOs and builders for invoices/transactions/retail/timeclock/report-card/subscription endpoints
- tests for pagination and legacy caveats (`list_transactions` pre-Aug 1 2019, `list_invoices` on/after Aug 1 2019)
- semantic conversion adapters or follow-up gap notes for retail/training/grooming domains

### 8. Webhook implementation

Deliverables:
- event enum and envelope DTOs
- HMAC verifier with constant-time comparison if feasible
- typestate or newtype that only exposes `VerifiedWebhook` after signature verification
- tests using doc-derived fixtures and negative signature tests
- retry/response-code helper semantics for receiver layer, without implementing a live HTTP server unless explicitly scoped

### 9. Integration review and AI-agent readiness brief

Deliverables:
- `docs/integrations/gingr/sdk-readiness-review.md`
- final tests/gates evidence
- “what this enables for NVA AI program” talking points:
  - safe read-only operational intelligence across locations
  - reservation/check-in/check-out/daycare/boarding/grooming context ingestion
  - owner/animal/reference-data enrichment
  - webhook-triggered agent workflows with signature gates
  - customer-portal event analytics boundary
  - clear separation between draft/recommendation agents and human-reviewed operational writes

---

## Execution constraints

- This workspace is currently not a git checkout; do not rely on git branches/worktrees.
- Existing board `pet-resort-service-domain-contracts` currently has code-mutating work in `/home/eran/code/pet-resort-agent-foundation`. Keep SDK code cards gated until that board is done or explicitly paused.
- Documentation/source-extraction cards may run immediately if they write distinct files under `docs/integrations/gingr/`.
- Code cards must follow semantic-code doctrine: failing semantic API tests first, RED verified, smallest domain/integration type surface, then focused tests and full `cargo test --workspace` when feasible.
- Do not perform live Gingr API calls unless the user later provides a sandbox subdomain/key and explicitly approves read-only probing.
- Never print or persist real API keys. Local test config must use fake/sentinel keys only.
