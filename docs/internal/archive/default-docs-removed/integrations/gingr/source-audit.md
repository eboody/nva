# Gingr SDK source audit

Audit date: 2026-06-10

Scope: validate the local Gingr developer corpus before SDK v0 implementation. This audit used only the public Gingr support site and the local inventory under `docs/integrations/gingr/`; no live Gingr tenant, API key, or customer data was used.

## Verdict

The local corpus is sufficient for SDK v0 documentation extraction and implementation planning.

Confidence: medium-high for the read API and webhook contract; medium for customer-portal JavaScript; low for any mutating or operational side effects.

Reasons:
- The public Developer Guide category resolves: https://support.gingrapp.com/hc/en-us/categories/25361341525645-Developer-Guide
- `manifest.json` records 32 crawled articles and one known fetch error.
- All 32 manifest article URLs returned HTTP 200 during this audit.
- The known stale URL still returns HTTP 404: https://support.gingrapp.com/hc/en-us/articles/26686883788685
- The canonical read API and webhook sources contain the SDK-v0-critical facts: base URL pattern, API-key auth, read-only boundary, endpoint list, webhook event enum, HMAC signature rule, response-code retry semantics, and payload examples.

Implementation guardrail: do not treat the `quick_checkin` and `receive_call` entries in the API functions article as SDK v0 read-only endpoints. They appear in the same public functions article but describe operational side effects. Keep them out of the v0 SDK surface unless a later human-reviewed task explicitly scopes a safe command boundary.

## Local source inventory

Canonical local files:
- `docs/integrations/gingr/README.md`
- `docs/integrations/gingr/manifest.json`
- `docs/integrations/gingr/articles/`

Inventory counts from `manifest.json`:
- total articles: 32
- total local article text characters: 66,761
- `customer_portal`: 7 articles
- `customer_portal_js`: 6 articles
- `integrations`: 2 articles
- `read_api`: 8 articles
- `webhooks`: 8 articles
- `unknown`: 1 article (`Sign in to Gingr`, redirects to auth when fetched publicly)
- fetch errors: 1 known 404 (`26686883788685`)

## Public link verification

Checked with public unauthenticated GET requests on 2026-06-10.

High-level result:
- Developer Guide category: HTTP 200
- 32 manifest article URLs: HTTP 200
- `https://support.gingrapp.com/hc/en-us/articles/25656686643213`: HTTP 200 but redirects to Zendesk auth; keep as non-SDK background only.
- `https://support.gingrapp.com/hc/en-us/articles/26686883788685`: HTTP 404; stale related article, not a v0 blocker based on current corpus coverage.

## SDK-v0-critical sources workers should cite

### API overview and read-only boundary

Use these as the canonical citations for public API posture:
- `docs/integrations/gingr/articles/26687020531853-api-feature-overview.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/26687020531853-API-Feature-Overview
  - confidence: high
  - cite for: public API exists to extract data; Gingr says the public API is read only; support/development teams do not provide implementation guidance.
- `docs/integrations/gingr/articles/27482358729101-api-feature-overview.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/27482358729101-API-Feature-Overview
  - confidence: medium
  - cite for: alternate/current Developer Guide API overview. Treat as corroborating, not primary, because the canonical plan already cites `26687020531853`.

### API keys, HTTPS JSON, permissions

Use this as the canonical auth/source-of-truth citation:
- `docs/integrations/gingr/articles/25721874372109-access-api-keys-how-to.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25721874372109-Access-API-Keys-How-To
  - confidence: high
  - cite for: all API requests require a user-account-based key; requests are HTTP over TLS/HTTPS; responses are JSON objects; endpoints are read-only; user must have `Can Access API`; API keys must not be shared and can be rotated.

Secondary API-key citation:
- `docs/integrations/gingr/articles/26817570653965-use-access-api-keys-how-to.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/26817570653965-Use-Access-API-Keys-How-To
  - confidence: medium
  - cite for: Developer Guide outline coverage, not primary auth semantics.

### API endpoint reference

Use this as the canonical endpoint catalog source:
- `docs/integrations/gingr/articles/25722122517517-gingr-api-functions-reference.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25722122517517-Gingr-API-Functions-Reference
  - confidence: high for listed request shapes; medium for detailed response schemas because examples are partial and inconsistent.
  - cite for: base URL pattern `https://{your_app}.gingrapp.com`, method/path list, required and optional params, form/query examples, 30-day reservation range, location-scoping caveats, invoice/transaction date caveats, pagination rules.

SDK-v0 safe read endpoints directly supported by this source:
- `GET /api/v1/get_locations`
- `GET /api/v1/reservation_types`
- `GET /api/v1/reservation_widget_data`
- `GET /api/v1/owners`
- `GET /api/v1/animals`

Additional read endpoints available for later expansion:
- reservations: `POST /api/v1/reservations`, `POST /api/v1/reservations_by_animal`, `POST /api/v1/reservations_by_owner`, `POST /api/v1/recently_cancelled_reservations`, `GET /api/v1/get_services_by_type`, `GET /api/v1/existing_reservation_estimate`, `GET /api/v1/back_of_house`
- owners/animals/forms: `POST /api/v1/new_modified_owners`, `POST /api/v1/authorize_owner`, `GET /forms/get_form`, `GET /api/v1/owner`, `GET /api/v1/custom_field_search`
- reference data: `GET /api/v1/get_species`, `GET /api/v1/get_breeds`, `GET /api/v1/get_vets`, `GET /api/v1/get_temperaments`, `GET /api/v1/get_immunization_types`, `GET /api/v1/get_animal_immunizations`
- commerce/labor/files: `GET /api/v1/get_all_retail_items`, `GET /api/v1/list_transactions`, `GET /api/v1/list_invoices`, `POST /api/v1/transaction`, `GET /api/v1/timeclock_report`, `GET /api/v1/report_card_files`, `GET /api/v1/get_subscription`, `GET /api/v1/get_subscriptions`, `GET /api/v1/get_feeding_info`, `GET /api/v1/get_medication_info`

Do not include in v0 as read-only SDK endpoints without a separate reviewed design:
- `GET /api/v1/quick_checkin` — checks in pet(s), and may create a reservation.
- `POST /api/v1/receive_call` — notifies Gingr of an incoming phone call and records it.

### Webhooks

Canonical webhook topic and setup sources:
- `docs/integrations/gingr/articles/26688281811597-webhooks-topic-outline.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/26688281811597-Webhooks-Topic-Outline
  - confidence: medium
  - cite for: Webhooks topic entry point.
- `docs/integrations/gingr/articles/25660732990477-activate-webhooks-how-to.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25660732990477-Activate-Webhooks-How-To
  - confidence: medium-high
  - cite for: webhook activation/configuration flow and signature-key setup.

Canonical webhook event/signature/retry source:
- `docs/integrations/gingr/articles/25660010986125-event-types-and-response-codes-reference.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25660010986125-Event-Types-and-Response-Codes-Reference
  - confidence: high
  - cite for: event enum, HMAC signature verification, and response-code semantics.

Webhook event enum from the docs:
- `check_in`
- `check_out`
- `checking_in`
- `checking_out`
- `email_sent`
- `owner_created`
- `owner_edited`
- `animal_created`
- `animal_edited`
- `incident_created`
- `incident_edited`
- `lead_created`

Webhook signature rule:
- SHA256 HMAC.
- Message is the concatenation of `webhook_type`, `entity_id`, and `entity_type`.
- Key is the configured signature key.
- SDK should expose a signature-verification gate/newtype before payload data can enter agent/domain workflows.

Webhook response semantics:
- HTTP 200: successfully received and processed; no retry.
- HTTP 403: receiver does not want the event; no retry.
- any other status: Gingr retries 10 times with increasing timeout durations.

Canonical webhook payload examples source:
- `docs/integrations/gingr/articles/25659829911565-event-data-structure-examples-reference.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25659829911565-Event-Data-Structure-Examples-Reference
  - confidence: high for envelope shape and sample fields; medium for full schema completeness.
  - cite for: common envelope fields `webhook_url`, `webhook_type`, `entity_id`, `entity_type`, `signature`, `entity_data`; email event additions `email_data` and `recipients`; reservation event note that `check_out` includes invoice data not shown in the sample.

### Customer Portal JavaScript boundary

Canonical topic/setup sources:
- `docs/integrations/gingr/articles/26687431397901-custom-css-javascript-topic-outline.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/26687431397901-Custom-CSS-JavaScript-Topic-Outline
  - confidence: medium
  - cite for: customer-facing custom CSS/JavaScript topic boundary.
- `docs/integrations/gingr/articles/25845902262285-add-custom-css-javascript-how-to.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25845902262285-Add-Custom-CSS-JavaScript-How-To
  - confidence: medium
  - cite for: custom code injection/setup location and scope.

Canonical JavaScript event sources:
- `docs/integrations/gingr/articles/25846095500557-javascript-events-emitted-reference.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25846095500557-JavaScript-Events-Emitted-Reference
  - confidence: medium-high
  - cite for: emitted customer-portal JavaScript event names.
- `docs/integrations/gingr/articles/25846282748045-listen-for-custom-javascript-events-reference.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25846282748045-Listen-for-Custom-JavaScript-Events-Reference
  - confidence: medium-high
  - cite for: how custom JavaScript listeners receive portal events.

Analytics examples, useful but not SDK-core:
- `docs/integrations/gingr/articles/25846389005325-google-tag-manager-adwords-analytics-in-javascript-events-reference.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25846389005325-Google-Tag-Manager-Adwords-Analytics-in-JavaScript-Events-Reference
  - confidence: medium
- `docs/integrations/gingr/articles/25846641444109-meta-pixel-in-javascript-events-reference.md`
  - public URL: https://support.gingrapp.com/hc/en-us/articles/25846641444109-Meta-Pixel-in-JavaScript-Events-Reference
  - confidence: medium

SDK boundary: customer-portal JavaScript is not the backend API. For the AI program, keep it to analytics capture, conversion tracking, lead-intent signals, and portal UX observation. Explicit review is required before collecting additional personal data, silently modifying customer-facing behavior, or auto-submitting portal actions.

## Full manifest link list

### customer_portal
- Access from a Web Browser (How-To): https://support.gingrapp.com/hc/en-us/articles/26757044893069-Access-from-a-Web-Browser-How-To
- Access from the Gingr for Pet Parents Mobile App (How-To): https://support.gingrapp.com/hc/en-us/articles/26757384207117-Access-from-the-Gingr-for-Pet-Parents-Mobile-App-How-To
- Get Started with the Customer Portal (Topic Outline): https://support.gingrapp.com/hc/en-us/articles/26810571027725-Get-Started-with-the-Customer-Portal-Topic-Outline
- Notifications Center Overview (Reference): https://support.gingrapp.com/hc/en-us/articles/26757899407117-Notifications-Center-Overview-Reference
- Reset Password for Pet Parents (How-To): https://support.gingrapp.com/hc/en-us/articles/26758082407437-Reset-Password-for-Pet-Parents-How-To
- Sign Legal Agreements in the Customer Portal (How To): https://support.gingrapp.com/hc/en-us/articles/33571861934093-Sign-Legal-Agreements-in-the-Customer-Portal-How-To
- Upload Vaccination Records (How-To): https://support.gingrapp.com/hc/en-us/articles/26757720199309-Upload-Vaccination-Records-How-To

### customer_portal_js
- Add Custom CSS & JavaScript (How-To): https://support.gingrapp.com/hc/en-us/articles/25845902262285-Add-Custom-CSS-JavaScript-How-To
- Google Tag Manager (Adwords/Analytics) in JavaScript Events (Reference): https://support.gingrapp.com/hc/en-us/articles/25846389005325-Google-Tag-Manager-Adwords-Analytics-in-JavaScript-Events-Reference
- JavaScript Events Emitted (Reference): https://support.gingrapp.com/hc/en-us/articles/25846095500557-JavaScript-Events-Emitted-Reference
- Listen for Custom JavaScript Events (Reference): https://support.gingrapp.com/hc/en-us/articles/25846282748045-Listen-for-Custom-JavaScript-Events-Reference
- Meta Pixel in JavaScript Events (Reference): https://support.gingrapp.com/hc/en-us/articles/25846641444109-Meta-Pixel-in-JavaScript-Events-Reference
- Set Up Owner's Map (How-To): https://support.gingrapp.com/hc/en-us/articles/26779465452685-Set-Up-Owner-s-Map-How-To

### integrations
- Customer Portal (Feature Overview): https://support.gingrapp.com/hc/en-us/articles/26690528840589-Customer-Portal-Feature-Overview
- Integrations (Topic Outline): https://support.gingrapp.com/hc/en-us/articles/26689830918669-Integrations-Topic-Outline

### read_api
- API (Feature Overview): https://support.gingrapp.com/hc/en-us/articles/27482358729101-API-Feature-Overview
- Access API Keys (How-To): https://support.gingrapp.com/hc/en-us/articles/25721874372109-Access-API-Keys-How-To
- Gingr API Functions (Reference): https://support.gingrapp.com/hc/en-us/articles/25722122517517-Gingr-API-Functions-Reference
- Gingr's API (Topic Outline): https://support.gingrapp.com/hc/en-us/articles/26817735212429-Gingr-s-API-Topic-Outline
- Google Maps API (Topic Outline): https://support.gingrapp.com/hc/en-us/articles/26785932585741-Google-Maps-API-Topic-Outline
- Set Up Address Lookup (How-To): https://support.gingrapp.com/hc/en-us/articles/26785597375885-Set-Up-Address-Lookup-How-To
- Set Up Address Lookup and Owner Map in Google Maps API (How-To): https://support.gingrapp.com/hc/en-us/articles/25722435154573-Set-Up-Address-Lookup-and-Owner-Map-in-Google-Maps-API-How-To
- Use & Access API Keys (How-To): https://support.gingrapp.com/hc/en-us/articles/26817570653965-Use-Access-API-Keys-How-To

### webhooks
- API (Feature Overview): https://support.gingrapp.com/hc/en-us/articles/26687020531853-API-Feature-Overview
- Activate Webhooks (How-To): https://support.gingrapp.com/hc/en-us/articles/25660732990477-Activate-Webhooks-How-To
- Custom CSS & JavaScript (Topic Outline): https://support.gingrapp.com/hc/en-us/articles/26687431397901-Custom-CSS-JavaScript-Topic-Outline
- Event Data Structure Examples (Reference): https://support.gingrapp.com/hc/en-us/articles/25659829911565-Event-Data-Structure-Examples-Reference
- Event Types and Response Codes (Reference): https://support.gingrapp.com/hc/en-us/articles/25660010986125-Event-Types-and-Response-Codes-Reference
- Get Started with API (Topic Outline): https://support.gingrapp.com/hc/en-us/articles/26687212732685-Get-Started-with-API-Topic-Outline
- Integrate Gingr with Slack via Zapier (How-To): https://support.gingrapp.com/hc/en-us/articles/25659213118733-Integrate-Gingr-with-Slack-via-Zapier-How-To
- Webhooks (Topic Outline): https://support.gingrapp.com/hc/en-us/articles/26688281811597-Webhooks-Topic-Outline

### non-SDK/background
- Sign in to Gingr: https://support.gingrapp.com/hc/en-us/articles/25656686643213

### stale/fetch error
- Known stale related article: https://support.gingrapp.com/hc/en-us/articles/26686883788685 — HTTP 404 during original crawl and during this audit.

## Public/private gaps

Public docs are enough for SDK v0 scaffolding and tests around request construction, redaction, endpoint typing, webhook verification, and fixture-derived payload parsing.

Known gaps that require private tenant docs, sandbox access, or later live read-only probing before production-grade completeness:
- Response schemas are partial and sample-oriented; many fields are provider-specific strings/booleans/timestamps and should remain quarantined behind DTOs until mapped deliberately.
- Error envelope/status-code behavior for read API endpoints is not fully specified in the public docs.
- Rate limits, pagination defaults, and maximum page sizes are only partially documented.
- Multi-location semantics depend on the API user's current logged-in/operating location for at least some reservation endpoints.
- `authorize_owner` accepts customer email/password and should be treated as sensitive/private-auth behavior, not a general safe read endpoint for agent workflows.
- Customer Portal JavaScript docs cover browser integration events, not backend API semantics.
- The public docs do not define a sandbox tenant, test key format, webhook replay mechanism, or deterministic fixture suite.

## Worker citation policy

Workers should cite exact public URLs and local article paths from this audit rather than citing only the category page. Preferred citations:
- API read-only/auth: `25721874372109` and `26687020531853`.
- API endpoint catalog: `25722122517517`.
- Webhook events/signature/retries: `25660010986125`.
- Webhook payload examples: `25659829911565`.
- Webhook setup: `25660732990477`.
- Portal JS emitted/listened events: `25846095500557` and `25846282748045`.

Workers should explicitly note that `26686883788685` is stale (404) if they reference crawl completeness. It does not block SDK v0 because the current corpus already covers the v0 API and webhook facts listed above.
