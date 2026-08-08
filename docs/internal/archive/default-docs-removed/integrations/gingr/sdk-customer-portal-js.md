# Gingr SDK customer-portal JavaScript boundary

Extraction date: 2026-06-10

Scope: document the customer-facing Customer Portal custom CSS/JavaScript integration boundary for SDK planning. This is not backend API documentation and does not authorize implementing portal JavaScript in the SDK. Sources are the local Gingr Developer Guide corpus under `docs/integrations/gingr/articles/`; no live Gingr tenant, API key, or customer data was used.

## Verdict

Customer Portal JavaScript belongs in the SDK documentation as a separate browser-snippet/event-observation boundary, not as part of the backend read API crate.

Recommended placement:
- Core SDK: do not include customer-portal JavaScript injection or browser automation in the backend read API surface.
- SDK docs: include this file as a boundary note so implementers do not confuse portal events with API endpoints or webhooks.
- Examples/snippets: future examples may include browser-side listener snippets for analytics/event forwarding, but only as optional customer-portal integration examples with explicit privacy and review gates.
- Agent workflows: treat portal JavaScript as an observational signal source. Any action that changes customer-facing behavior, submits forms, or collects extra personal data requires human review before deployment.

## Source map

Canonical local sources:
- `docs/integrations/gingr/articles/26687431397901-custom-css-javascript-topic-outline.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/26687431397901-Custom-CSS-JavaScript-Topic-Outline
  - Use for: topic boundary; Gingr says custom CSS/JavaScript is embedded into the customer-facing application for color/font customization and advertising campaign trackers.
- `docs/integrations/gingr/articles/25845902262285-add-custom-css-javascript-how-to.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/25845902262285-Add-Custom-CSS-JavaScript-How-To
  - Use for: setup location and field semantics: Admin -> Custom Configurations; Customer app CSS; Customer app JS/Customer app footer; script/style tag behavior.
- `docs/integrations/gingr/articles/25846095500557-javascript-events-emitted-reference.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/25846095500557-JavaScript-Events-Emitted-Reference
  - Use for: emitted event names and basic event meaning.
- `docs/integrations/gingr/articles/25846282748045-listen-for-custom-javascript-events-reference.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/25846282748045-Listen-for-Custom-JavaScript-Events-Reference
  - Use for: document-level listener shape and `e.detail` payload hints.
- `docs/integrations/gingr/articles/25846389005325-google-tag-manager-adwords-analytics-in-javascript-events-reference.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/25846389005325-Google-Tag-Manager-Adwords-Analytics-in-JavaScript-Events-Reference
  - Use for: GTM/GA4 conversion tracking example and Customer Portal 2.0 caveat.
- `docs/integrations/gingr/articles/25846641444109-meta-pixel-in-javascript-events-reference.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/25846641444109-Meta-Pixel-in-JavaScript-Events-Reference
  - Use for: Meta Pixel conversion example.
- `docs/integrations/gingr/articles/26690528840589-customer-portal-feature-overview.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/26690528840589-Customer-Portal-Feature-Overview
  - Use for: customer portal capability scope: pet parents can update information, request reservations, upload immunizations, buy packages, make payments, order retail items, and more.
- `docs/integrations/gingr/articles/26757044893069-access-from-a-web-browser-how-to.md`
  - Public URL: https://support.gingrapp.com/hc/en-us/articles/26757044893069-Access-from-a-Web-Browser-How-To
  - Use for: portal URL pattern `https://businessname.portal.gingrapp.com/` and browser access context.

## Boundary definition

This integration runs in Gingr's customer-facing portal, not in the backend API.

Key facts from the docs:
- Gingr allows embedded custom CSS and JavaScript in the customer-facing application.
- The common documented use cases are visual customization and advertising campaign trackers.
- Setup happens through Gingr admin configuration fields, not through API endpoints:
  - `Customer app CSS`
  - `Customer app JS`
  - `Customer app footer`
- Gingr automatically includes a `<style>` tag for the Customer App CSS field and a `<script>` tag for the Customer App JS field; code that already contains a `<script>` tag belongs in the Customer App Footer field.
- Gingr states its support and development teams do not provide implementation guidance for custom CSS/JavaScript.
- Customer Portal 2.0 documentation says the `Customer App CSS Field` and `Customer App JS field` are non-functional for GTM setup, and the example uses `Customer app footer` instead.
- GTM testing is performed against the customer portal URL, usually `https://<app>.portal.gingrapp.com`.
- The Customer Portal is the pet-parent interface for account creation, reservation requests, immunization uploads, legal agreements, payments, retail orders, and other customer actions.

SDK implication: a Rust backend SDK should not expose this as `GingrClient` methods. If captured at all, represent it as documentation, typed event names, and optional browser snippet examples that a human installs into the portal/admin configuration.

## Emitted portal events

The local corpus documents three customer-portal JavaScript events:

| Event | Meaning in docs | Observability note |
| --- | --- | --- |
| `reservation_created` | Emitted when a reservation or appointment request has been made. | Listener examples indicate `e.detail.reservation_ids` contains an object with animal ID and reservation IDs that were just created. |
| `owner_created` | Emitted when a customer registers a new account. | Listener examples indicate `e.detail` contains user-supplied form data. Treat this as personal data. |
| `lead_created` | Emitted when a customer submits a lead form. | Useful as lead-intent signal; docs do not define a payload schema. |

Documented listener shape:

```javascript
document.addEventListener('reservation_created', function (e) {
  // add conversion code here
  // e.detail.reservation_ids contains animal and reservation IDs that were just created
}, false);

document.addEventListener('owner_created', function (e) {
  // add conversion code here
  // e.detail contains user-supplied form data submitted on the form
}, false);
```

Do not infer a complete schema from these snippets. The docs provide examples and comments, not a stable typed contract.

## Analytics and conversion examples

The docs show this boundary primarily as analytics/conversion tracking:
- GTM/GA4: initialize GTM/GA4, create custom event triggers for `owner_created`, `reservation_created`, or `lead_created`, then push event data into `window.dataLayer` from a `document.addEventListener(...)` handler.
- Meta Pixel: initialize the pixel and call `fbq('track', 'Lead', {})` from a `reservation_created` listener.

Important caveats:
- These examples run in the customer-facing portal.
- GTM/Meta identifiers are third-party analytics configuration, not Gingr API credentials.
- The examples are not backend SDK behavior and should not be hidden inside server-side code.
- `owner_created` details can include user-supplied form data, so forwarding raw `e.detail` to analytics or AI systems can create privacy and consent issues.

## Safe AI-program uses

Safe by default when implemented as explicitly reviewed browser snippets or downstream analytics processing, not as automatic customer-facing behavior changes:

- Conversion tracking:
  - Count reservation request completions via `reservation_created`.
  - Count new account registrations via `owner_created`.
  - Count lead form submissions via `lead_created`.
- Funnel and UX observation:
  - Compare event counts against traffic/session data to find drop-off points.
  - Monitor aggregate event timing and conversion rates after reviewed portal changes.
- Lead-intent signals:
  - Route `lead_created` counts or sanitized lead metadata into reporting dashboards.
  - Trigger internal follow-up recommendations, not automated customer contact, unless a later card scopes and reviews that workflow.
- Campaign attribution:
  - Forward minimal event names and campaign IDs to GTM/GA4/Meta where configured by the business.
- SDK developer guidance:
  - Publish typed event-name constants in docs or optional examples so implementers use canonical strings.
  - Provide redaction guidance for event payloads before analytics/AI ingestion.

Safe-data posture:
- Prefer event name, timestamp, portal/app identifier, campaign/source metadata, and coarse aggregate counts.
- Treat `owner_created` and `lead_created` payloads as personal data unless proven otherwise.
- Avoid forwarding raw `e.detail` to third parties or model prompts.

## Unsafe uses requiring explicit review

These require a separate reviewed design, privacy/legal approval where applicable, and customer-facing QA before deployment:

- Silently modifying customer-facing behavior:
  - Changing reservation flows, hiding fields, bypassing validations, modifying prices/promotions, or altering legal/payment/immunization UI behavior.
- Auto-submitting or automating portal actions:
  - Creating accounts, submitting leads, requesting reservations, uploading records, accepting agreements, making payments, or ordering retail items on behalf of customers.
- Collecting more personal data than intended:
  - Capturing full form payloads, owner contact details, pet details, payment context, immunization records, legal agreement status, or free-text notes without explicit data-minimization review.
- Sending raw portal event payloads into AI/model workflows:
  - Especially `owner_created` `e.detail`, which docs describe as user-supplied form data.
- Interfering with third-party scripts or consent:
  - Injecting trackers without consent controls, overriding analytics consent state, or duplicating third-party pixels in ways that produce unintended tracking.
- Treating browser events as authoritative operational state:
  - Portal events are frontend signals. Backend read API data and verified webhooks are better sources for operational decisions.
- Native app assumptions:
  - GTM docs state this feature is not available for customer portal native apps; do not promise equivalent behavior in the Gingr for Pet Parents native app.

## SDK vs browser-snippet placement

### Keep out of core backend SDK

Do not implement:
- Admin writes that install portal JavaScript.
- Browser DOM manipulation in the Rust API client.
- Event listeners as part of `GingrClient` request/response modules.
- Automatic forwarding of portal payloads into agent memory, prompts, or commands.

### Allow in docs/examples

Acceptable future documentation/examples:
- An `examples/browser/customer_portal_events.md` or equivalent docs page showing minimal listener snippets.
- A typed list of event names for humans writing snippets:
  - `reservation_created`
  - `owner_created`
  - `lead_created`
- Redaction and minimization examples before event forwarding.
- A clear statement that snippets must be installed manually in Gingr admin configuration by an authorized operator.

### Optional bridge pattern, if later scoped

A later reviewed card may define a browser-to-backend bridge with these constraints:
- Event allowlist restricted to documented event names.
- Payload allowlist/redaction per event; no raw `owner_created` or `lead_created` forwarding by default.
- Explicit consent/compliance requirements for third-party analytics destinations.
- Receiver endpoint separated from the Gingr read API client and from webhook verification logic.
- No customer-facing behavior mutation; observation only.
- Test fixtures use synthetic event payloads only.

## Relationship to webhooks and read API

Do not collapse these three boundaries:

| Boundary | Runs where | Trust/semantics | SDK treatment |
| --- | --- | --- | --- |
| Read API | Server-to-Gingr HTTPS JSON endpoints | Read-only backend data access with API key; endpoint request/response contracts. | Core SDK request builders, redacted transport, DTO quarantine, semantic mapping. |
| Webhooks | Gingr-to-receiver HTTP callbacks | Server-side event delivery with HMAC signature and retry semantics. | Separate webhook module with explicit verification gate before domain/agent workflows. |
| Customer Portal JavaScript | Browser/customer-facing portal | Frontend event observations and customer-facing script/CSS customization. | Documentation and optional browser snippets only; no core backend methods. |

Portal JavaScript events can complement webhooks and API reads for analytics, but they are not substitutes for verified webhook payloads or backend read-model reconciliation.

## Implementation guardrails for future cards

If a future card adds examples or bridge code, it should include tests/checks for:
- documented event-name allowlist only;
- no raw `e.detail` logging by default;
- no API keys, GTM IDs, pixel IDs, or real tenant domains committed in examples;
- synthetic fixtures only;
- clear labels that snippets are customer-facing and must be reviewed before deployment;
- explicit distinction between observation, recommendation, and customer-affecting action.

No implementation is performed by this card.
