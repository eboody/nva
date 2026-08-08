# Gingr SDK webhook contract catalog

Source category: https://support.gingrapp.com/hc/en-us/categories/25361341525645-Developer-Guide

Primary sources:
- Webhooks topic outline: https://support.gingrapp.com/hc/en-us/articles/26688281811597-Webhooks-Topic-Outline
- Activate Webhooks: https://support.gingrapp.com/hc/en-us/articles/25660732990477-Activate-Webhooks-How-To
- Event types, response codes, and HMAC rule: https://support.gingrapp.com/hc/en-us/articles/25660010986125-Event-Types-and-Response-Codes-Reference
- Event data structure examples: https://support.gingrapp.com/hc/en-us/articles/25659829911565-Event-Data-Structure-Examples-Reference

This document is SDK-ready extraction only. It does not authorize live Gingr calls and does not contain real customer payloads or real signature keys.

## Operational setup contract

Gingr sends webhook HTTP(S) requests to the URL(s) configured in the app under `Admin » Custom Configurations`. The `Webhook URL` field may contain multiple comma-separated URLs. The same configuration screen also contains a `Webhook Signature Key` field, which should be set to a random secret controlled by the receiving system.

Third-party integration products such as Zapier/IFTTT can receive these hooks, but the SDK receiver should model Gingr itself as the source provider and should treat all inbound webhooks as untrusted until signature verification succeeds.

## Event enum

The SDK should expose a closed event enum for currently documented webhook event names, plus an explicit unknown/raw variant for forward compatibility if the receiver chooses not to fail parsing on future Gingr events.

| Technical event name | Human readable name | Documented meaning | Suggested SDK enum |
| --- | --- | --- | --- |
| `check_in` | Reservation checked in | Triggered when a reservation is checked in. | `CheckIn` |
| `check_out` | Reservation checked out | Triggered when a reservation is checked out. | `CheckOut` |
| `checking_in` | Reservation checking in | Triggered when a user displays intent to check a reservation in, including pet parent text `HERE` or employee dashboard check-in click. | `CheckingIn` |
| `checking_out` | Reservation checking out | Triggered when a reservation is added to the shopping cart. | `CheckingOut` |
| `email_sent` | Email sent | Triggered when a system-generated email is sent from Gingr. | `EmailSent` |
| `owner_created` | Owner created | Triggered when a new owner record is created from the customer portal or by an employee in the app. | `OwnerCreated` |
| `owner_edited` | Owner edited | Triggered when an owner record is updated from the customer portal or by an employee in the app. | `OwnerEdited` |
| `animal_created` | Animal created | Triggered when a new animal record is created from the customer portal or by an employee in the app. | `AnimalCreated` |
| `animal_edited` | Animal edited | Triggered when an animal record is updated from the customer portal or by an employee in the app. | `AnimalEdited` |
| `incident_created` | Incident created | Triggered when an employee creates a new incident for an animal in the app. | `IncidentCreated` |
| `incident_edited` | Incident edited | Triggered when an employee updates an existing incident for an animal in the app. | `IncidentEdited` |
| `lead_created` | Lead created | Triggered when a pet parent fills out a lead form using the embeddable lead capture form or customer portal. | `LeadCreated` |

Implementation note: do not infer a business action from event name alone. The verified envelope should still be mapped through event-specific semantic adapters before any domain workflow sees it.

## Common envelope

Documented webhook requests are JSON objects with these common fields:

| Field | Type observed in docs | Meaning | SDK guidance |
| --- | --- | --- | --- |
| `webhook_url` | string | URL Gingr is sending the request to. | Parse as opaque provider-observed URL or validated URL; trim incidental documentation whitespace if fixture-derived. |
| `webhook_type` | string enum | Technical event name. | Parse into `WebhookEventType`; preserve raw string for unknown events or diagnostics. |
| `entity_id` | string or number | ID associated with the entity. Reservation example is a string; email example is a number. | Accept string-or-integer at the boundary, normalize to a non-empty string for signature input and semantic IDs. |
| `entity_type` | string enum-ish | Entity type associated with the webhook. Examples include `reservation` and `owner`. | Parse into `WebhookEntityType` with an unknown/raw variant. |
| `signature` | lowercase hex string | SHA256 HMAC digest supplied by Gingr. | Decode/validate as 32-byte HMAC-SHA256 digest or compare against expected lowercase hex with constant-time equality. |
| `entity_data` | object | Entity payload associated with `entity_type` and `entity_id`. | Keep as raw provider DTO until verified, then map through entity/event-specific adapters. |

`email_sent` additionally includes:

| Field | Type observed in docs | Meaning | SDK guidance |
| --- | --- | --- | --- |
| `email_data` | object | Data for the sent email, including subject and HTML value. | Treat HTML body as untrusted provider content; sanitize before display or downstream text use. |
| `recipients` | array of objects | Email recipients with `name` and `email`. | Treat as PII; redact in logs/fixtures unless explicitly working with approved test accounts. |

## Entity types

The examples explicitly show:

| Entity type | Used by | Observed payload shape |
| --- | --- | --- |
| `reservation` | Reservation events, e.g. `check_out` | Combined reservation, animal, owner, vet, service/type, location, feeding, pricing, and timestamp fields. |
| `owner` | `email_sent` example | Owner profile fields plus email metadata and recipients. |

The event catalog implies additional domain entities for owner, animal, incident, and lead events, but the local documentation does not include full payload examples for all of them. The SDK should therefore use a typed envelope plus raw `entity_data` quarantine, then add event/entity-specific DTOs only when fixtures or integration tests prove the shape.

Suggested entity enum:

- `Reservation`
- `Owner`
- `Animal`
- `Incident`
- `Lead`
- `Unknown(String)`

## Signature verification

Gingr documents a `signature` property on every request. The signature is the output of SHA256 HMAC where:

- key: the configured `Webhook Signature Key`
- message: `webhook_type` + `entity_id` + `entity_type`, concatenated with no separators
- output: hex-encoded SHA256 HMAC digest

For example, for an envelope with:

```text
webhook_type = check_out
entity_id = 76390
entity_type = reservation
message = check_out76390reservation
```

The verifier computes:

```text
hex(HMAC_SHA256(signature_key, "check_out76390reservation"))
```

SDK requirements:

1. Normalize `entity_id` to exactly the same textual value that Gingr signs. Because examples show both string and numeric JSON forms, parse `entity_id` as a string-or-integer boundary value and convert integers to their base-10 decimal string representation without whitespace.
2. Do not include separators, JSON serialization, `webhook_url`, `signature`, or `entity_data` in the HMAC message.
3. Require the configured key to be a secret/newtype such as `WebhookSignatureKey`; never log it.
4. Compare the computed signature to the supplied signature with constant-time equality where available.
5. Return structured verification errors for missing fields, unsupported entity-id representation, malformed signature hex, and signature mismatch.

## Typestate boundary: unverified payloads cannot drive workflows

Webhook JSON is external input even when it appears to come from Gingr. The SDK should make this impossible to misuse:

```text
Raw HTTP body
  -> UnverifiedWebhookEnvelope
  -> verify(signature_key)
  -> VerifiedWebhookEnvelope
  -> event/entity semantic mapper
  -> domain event candidate or agent workflow input
```

Rules:

- `UnverifiedWebhookEnvelope` may expose only parsing metadata and verification inputs.
- `entity_data`, `email_data`, and `recipients` must not be passed into domain or agent workflows until the envelope is verified.
- `VerifiedWebhookEnvelope` should carry the verified event type, normalized entity id, entity type, and raw provider payload for explicit mapping.
- Agent workflows should consume semantic domain events such as “reservation checked in” only after both signature verification and event/entity mapping succeed.
- Unknown events may be acknowledged or rejected by policy, but should not silently enter automation.

## Response-code and retry semantics

Gingr expects the receiver to signal delivery handling through HTTP status codes:

| Receiver status | Gingr behavior | SDK receiver policy candidate |
| --- | --- | --- |
| `200` | Webhook was successfully received and processed. Gingr will not retry. | Return only after the receiver has durably accepted the verified event, or after intentionally accepting a harmless/ignored event. |
| `403` | Application does not want this event. Gingr will not retry. | Use for intentional permanent rejection, such as unknown event disallowed by policy or signature failure that should not be retried. |
| Any other status | Error happened. Gingr retries 10 times with increasing timeout durations. | Use for transient receiver failures, storage outages, or processing errors where retry is desirable. |

Receiver helper API suggestion:

- `WebhookAck::Processed` -> HTTP 200
- `WebhookAck::RejectedPermanently` -> HTTP 403
- `WebhookAck::RetryableFailure` -> HTTP 500 or a caller-selected retryable status

Do not return `200` merely because JSON parsed. If a workflow must be asynchronous, persist the verified event to a durable inbox first, then return `200`.

## Payload-shape caveats from examples

The examples are documentation samples, not a complete schema. The SDK should preserve provider wire semantics instead of over-normalizing too early.

Observed caveats:

- `entity_id` appears as both a JSON string (`"76390"`) and JSON number (`5917`).
- Many IDs and booleans are encoded as strings: examples include `animal_id`, `fixed`, `vip`, `location_id`, `opt_out_email`, and `allow_online_login`.
- Timestamps are often Unix timestamps encoded as strings, while reservation examples also include ISO strings such as `start_date_iso` and `end_date_iso`.
- Monetary values and rates are strings, e.g. `current_balance`, `base_rate`, `final_rate`, and `payment_amount`.
- Nullable fields are common, and empty strings also carry absence-like meaning in some fields.
- `entity_data` can contain HTML-bearing fields such as notes and email body data; treat these as untrusted text/HTML.
- The `check_out` event may include invoice data beyond the reservation example; the shown reservation example is not exhaustive.
- Payloads include PII (names, addresses, emails, phones, location data) and operational notes; logs and fixtures must be sanitized.

## Fixture candidates

Sanitized fixture candidates live under `docs/integrations/gingr/fixtures/webhooks/`:

- `reservation-check-out.json` — reservation-style `check_out` envelope, sanitized from the docs and signed with fake key `test-webhook-signature-key`.
- `email-sent.json` — `email_sent` owner/email envelope, sanitized from the docs and signed with fake key `test-webhook-signature-key`.
- `README.md` — explains fixture scope, sanitization, and recomputation of signatures.

These fixtures are for parser/verifier tests only. They are not proof of complete schemas for every event/entity pair.

## Minimal Rust type surface implied by this contract

Suggested module-level types for the future SDK implementation:

```text
WebhookSignatureKey(secret)
WebhookSignature(hex digest)
WebhookEventType::{CheckIn, CheckOut, CheckingIn, CheckingOut, EmailSent, OwnerCreated, OwnerEdited, AnimalCreated, AnimalEdited, IncidentCreated, IncidentEdited, LeadCreated, Unknown(String)}
WebhookEntityType::{Reservation, Owner, Animal, Incident, Lead, Unknown(String)}
WebhookEntityId(String)
UnverifiedWebhookEnvelope
VerifiedWebhookEnvelope
WebhookVerificationError
WebhookAck::{Processed, RejectedPermanently, RetryableFailure}
```

The important architectural constraint is semantic: raw Gingr payloads remain boundary data until the signature gate and mapping layer produce domain-specific events.
