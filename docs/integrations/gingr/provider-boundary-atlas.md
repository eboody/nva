---
title: "Gingr provider boundary"
slug: "gingr-provider-boundary"
family: "provider-boundaries-and-source-evidence"
status: "draft"
audience: ["general-manager", "regional-ops", "docs-writer", "engineer"]
plain_english_definition: "The Gingr provider boundary is the set of NVA code and docs that reads Gingr evidence, keeps Gingr's wire data quarantined, and promotes only reviewed, explicitly mapped facts into NVA domain candidates."
primary_labor_problem: "Reduces repeated source reconciliation by making clear what Gingr says, what NVA derives from it, and which actions remain read-only, draft-only, or human-approved."
source_of_record: "Gingr API/webhook evidence for provider facts; NVA domain/app/storage contracts for NVA decisions and workflow outcomes."
authoritative_human_role: "front desk lead, general manager, or integration owner depending on the fact or exception"
workflow_links: ["booking-triage", "checkout-completion", "daily-care-updates", "data-quality-hygiene", "manager-daily-brief"]
source_paths:
  - "integrations/gingr/src/config.rs"
  - "integrations/gingr/src/transport.rs"
  - "integrations/gingr/src/endpoint/mod.rs"
  - "integrations/gingr/src/response.rs"
  - "integrations/gingr/src/dto/mod.rs"
  - "integrations/gingr/src/mapping/mod.rs"
  - "integrations/gingr/src/webhook.rs"
  - "docs/integrations/gingr/source-inventory.md"
  - "docs/integrations/gingr/sdk-endpoint-catalog.md"
  - "docs/integrations/gingr/sdk-webhooks.md"
rustdoc_contracts:
  - "gingr::config::{Subdomain, BaseUrl, ApiKey, Provider, Client}"
  - "gingr::endpoint::{Request, Date, IsoDate, DateRange, Method}"
  - "gingr::transport::{Client, RequestParts, RedactedRequest, Transport}"
  - "gingr::response::{Raw, Envelope, OwnerRecord, AnimalRecord, ReservationRecord, ReferenceRecord}"
  - "gingr::dto::{ProviderSurface, retail::Item}"
  - "gingr::mapping::{customer, pet, retail, Error, ProviderField}"
  - "gingr::webhook::{Envelope, Verified, EventType, EntityType, Ack, SignatureKey}"
glossary_links:
  - "../../glossary-source-data-terms.md#domainsourcesystemgingr-gingr"
  - "../../glossary-source-data-terms.md#source-of-record"
  - "../../glossary-source-data-terms.md#provider-record"
  - "../../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence"
  - "../../glossary-architecture-terms.md#dto"
  - "../../glossary-architecture-terms.md#adapter"
  - "../../glossary-workflow-state-terms.md#draft"
  - "../../glossary-workflow-state-terms.md#review-gate"
  - "../../glossary-workflow-state-terms.md#blocked-action"
allowed_action_summary: "read Gingr evidence, build redacted requests, parse responses/webhooks, map selected provider fields into review candidates, draft staff-facing work, and record source-backed outcomes"
blocked_action_summary: "no autonomous provider writes, source deletion, customer sends, check-in/out or schedule mutations, payment movement, safety approvals, or secret-dependent live side effects"
outcome_fields: ["source ref", "provider endpoint/event", "mapping candidate", "review disposition", "blocked-action reason", "redaction evidence", "outcome/labor record"]
---

# Gingr provider boundary atlas

This draft family entry explains the Gingr provider boundary for non-coders and maintainers. It uses the [entity atlas page template](../../design/entity-atlas-page-template.md) but groups several small technical DTO, mapping, endpoint, transport, response, webhook, and source-inventory concepts into one family page because most of them are not independent resort entities.

The short version: [Gingr](../../glossary-source-data-terms.md#domainsourcesystemgingr-gingr) can say what the provider observed in its API responses and webhook payloads. NVA can derive source-backed candidates only after the integration code validates request shape, quarantines raw provider payloads, and maps a specific field into a domain or workflow concept. Markdown docs orient reviewers; the source files, Rust module contracts, and tests remain the behavior authority.

## 1. Plain-English pet-resort definition

The Gingr provider boundary is the fence between Gingr's external records and NVA's own resort workflows. Gingr records can provide source evidence about owners, pets, reservations, retail items, reference data, timeclock reports, report-card files, and webhook events, but they are not automatically NVA truth.

For example, a Gingr `OwnerRecord` may contain a first name, last name, email, and phone number. NVA may use those fields to produce a customer contact candidate for staff review. That candidate is not proof of legal identity, customer outreach approval, or permission to write back to Gingr.

## 2. Purpose: labor-cost or safety problem

This page helps front desk leads, managers, regional reviewers, docs writers, and engineers avoid repetitive source reconciliation and unsafe automation by naming:

- what Gingr says as provider evidence;
- what NVA derives through explicit mapping code;
- which fields remain raw DTOs or unknown provider payloads;
- which workflow results are read-only evidence, draft-only recommendations, or reviewed outcomes;
- which actions must stay blocked unless a separate approved tool and human gate exists.

The safe labor outcome is a source-backed packet, queue, candidate, or draft that staff can review faster. It is not an autonomous customer send, check-in/out, booking mutation, payment action, or source-system write.

## 3. Atlas family entries

These are the family entries this page covers. Give one of them a separate atlas page only if a non-coder reviewer needs to understand it independently to make a source, safety, labor, or approval decision.

| Entry | Plain-English meaning | Primary source paths | What Gingr says | What NVA derives |
| --- | --- | --- | --- | --- |
| Provider config | The typed tenant URL, API key wrapper, and provider label used before any request can be sent. | [`config.rs`](../../../integrations/gingr/src/config.rs) | A tenant/app subdomain, API key, and Gingr installation identity supplied at the process boundary. | A validated `BaseUrl`, redacted `ApiKey`, and provider label for audit/request capture. |
| Endpoint request builders | Secret-free request descriptions for Gingr API reads. | [`endpoint/mod.rs`](../../../integrations/gingr/src/endpoint/mod.rs), endpoint submodules | Method, path, parameter names, date formats, location filters, and endpoint quirks from the Gingr docs. | Typed request parts that can be inspected, redacted, tested, and only then sent by transport. |
| Transport and request capture | HTTP mechanics plus log-safe request diagnostics. | [`transport.rs`](../../../integrations/gingr/src/transport.rs) | The request must go to Gingr over HTTPS with a `key` parameter. | A `RequestParts`/`RedactedRequest` boundary that inserts secrets only at send time and hides sensitive values from diagnostics. |
| Raw response wrappers | The quarantine for HTTP status, body bytes, provider envelopes, and unknown fields. | [`response.rs`](../../../integrations/gingr/src/response.rs) | JSON response bodies, provider error text, `success` flags, owner/animal/reservation/reference rows, and extra fields. | Boundary DTOs that preserve evidence without turning provider flags or unknown fields into NVA decisions. |
| DTO surfaces | Provider-shaped records that are known enough to deserialize or intentionally marked as gaps. | [`dto/mod.rs`](../../../integrations/gingr/src/dto/mod.rs), [`dto/retail.rs`](../../../integrations/gingr/src/dto/retail.rs), [`dto/grooming.rs`](../../../integrations/gingr/src/dto/grooming.rs), [`dto/training.rs`](../../../integrations/gingr/src/dto/training.rs) | Retail item fields and service areas where Gingr has endpoints but no stable typed DTO here yet. | Product candidates where fields are sufficient; explicit `ProviderSurface` gaps where mapping would be unsafe. |
| Mapping adapters | Explicit transformations from Gingr DTOs into reviewable domain candidates. | [`mapping/mod.rs`](../../../integrations/gingr/src/mapping/mod.rs), [`mapping/customer.rs`](../../../integrations/gingr/src/mapping/customer.rs), [`mapping/pet.rs`](../../../integrations/gingr/src/mapping/pet.rs), [`mapping/retail.rs`](../../../integrations/gingr/src/mapping/retail.rs) | Provider owner, animal, and retail fields. | Customer contact, pet name, and retail product candidates, or semantic mapping errors when required provider fields are missing/invalid. |
| Webhook verification boundary | Inbound webhook parsing, HMAC verification, event/entity normalization, and acknowledgement policy. | [`webhook.rs`](../../../integrations/gingr/src/webhook.rs), [`sdk-webhooks.md`](sdk-webhooks.md) | Event name, entity type/id, signature, entity data, optional email data/recipients. | A verified event envelope that may later be mapped; unverified payloads cannot drive workflows. |
| Source inventory and read model | The documented source corpus and catalog of read endpoints. | [`source-inventory.md`](source-inventory.md), [`sdk-endpoint-catalog.md`](sdk-endpoint-catalog.md), [`bi-read-model-contract.md`](bi-read-model-contract.md), [`source-audit.md`](source-audit.md) | Which Gingr docs, endpoints, fixtures, and response caveats are known. | A reviewed inventory of read-only evidence and gaps for future mapping, BI/read-model, and workflow work. |
| Provider authority boundary | The rule that provider evidence is not business approval. | This page plus source/tests above | Gingr can report provider records, events, statuses, and payload fields. | NVA decides policy, workflow disposition, blocked actions, and outcome capture through domain/app/storage contracts and human review. |

## 4. Workflows where it appears

| Workflow | How the Gingr boundary appears | Safe workflow result |
| --- | --- | --- |
| Booking triage | Reservation, owner, pet, immunization/reference, and location evidence can support a readiness packet. | Reviewable packet or draft; no provider confirmation/write, schedule mutation, or customer send. |
| Checkout completion | Reservation/check-out evidence and commerce/payment-sensitive records may identify exception candidates. | Staff-facing exception queue or reviewed outcome; no autonomous payment/refund/discount movement. |
| Daily care updates | Report-card files, reservation context, owner/pet contact evidence, and webhook events may support update drafts. | Draft or queue item for staff approval; no autonomous parent communication. |
| CRM/retention and grooming/training follow-up | Owner, animal, reservation, service, retail, grooming/training gap, and email event evidence may support follow-up candidates. | Recommendation or draft; no marketing/customer send without approved sender and suppression rules. |
| Data quality hygiene | Provider IDs, raw payload refs, unknown fields, and mapping errors identify duplicate/missing/conflicting source facts. | Cleanup candidate and disposition record; no source-data deletion or hiding. |
| Manager daily brief / BI read model | Read-only endpoint inventory and mapped candidates help summarize labor-impacting queues. | Source-backed brief item or reporting signal; not staffing, payroll, or policy authority. |

## 5. Relationships and adjacency

```text
Gingr docs / API / webhook
  -> config + endpoint request builder
  -> transport request capture + redaction
  -> raw response or raw webhook envelope
  -> DTO / provider payload quarantine
  -> mapping adapter or explicit mapping gap
  -> NVA domain/workflow candidate
  -> staff review gate + outcome/audit record
```

Important adjacencies:

- Provider IDs such as owner, animal, reservation, retail item, location, subscription, package, user, and transaction IDs are source evidence, not NVA domain IDs by themselves.
- Provider statuses, booleans, money strings, timestamps, nulls, and empty strings stay provider DTO details until a mapping adapter names the NVA meaning.
- Unknown fields are preserved for audit and future mapping, not used silently for policy.
- Webhook payload data is untrusted until the signature gate passes.
- Response and webhook records may contain PII, payment-sensitive details, HTML, notes, recipients, or credentials; docs and diagnostics must stay sanitized.

## 6. Contracts and source/Rustdoc links

| Contract type | Link or path | What the writer should verify |
| --- | --- | --- |
| Atlas template | [Entity atlas page template](../../design/entity-atlas-page-template.md) | Required boundary language, authority sections, allowed/blocked action wording, and DTO merge guidance. |
| Provider source inventory | [Gingr SDK source inventory](source-inventory.md) | Which articles were used and whether a claim comes from Gingr docs or later NVA design. |
| Endpoint catalog | [Gingr SDK read-only endpoint catalog](sdk-endpoint-catalog.md) | Which endpoints are included, excluded, read-only, sensitive, location-scoped, paginated, or date-constrained. |
| Webhook contract catalog | [Gingr SDK webhook contract catalog](sdk-webhooks.md) | Event/entity names, HMAC rule, response-code policy, and fixture scope. |
| SDK architecture | [Gingr Rust SDK architecture](sdk-architecture.md) | Architectural intent: provider boundary, DTO quarantine, mapping into domain candidates, no side-effecting agent commands. |
| Config source | [`integrations/gingr/src/config.rs`](../../../integrations/gingr/src/config.rs) | Validated subdomain/base URL, redacted API key, provider label, config errors. |
| Endpoint source | [`integrations/gingr/src/endpoint/mod.rs`](../../../integrations/gingr/src/endpoint/mod.rs) and submodules | Request builders, date/range/pagination constraints, sensitive lookup fields, endpoint paths. |
| Transport source | [`integrations/gingr/src/transport.rs`](../../../integrations/gingr/src/transport.rs) | API key insertion, request capture, redaction, mock vs HTTP transport boundary. |
| Response source | [`integrations/gingr/src/response.rs`](../../../integrations/gingr/src/response.rs) | Raw response quarantine, envelope shape, owner/animal/reservation/reference DTOs, unknown field preservation. |
| DTO source | [`integrations/gingr/src/dto/mod.rs`](../../../integrations/gingr/src/dto/mod.rs) and child modules | Provider DTOs modeled today and surfaces intentionally left as mapping gaps. |
| Mapping source | [`integrations/gingr/src/mapping/mod.rs`](../../../integrations/gingr/src/mapping/mod.rs) and child modules | What provider fields can be promoted into candidates and what missing/invalid fields block mapping. |
| Webhook source | [`integrations/gingr/src/webhook.rs`](../../../integrations/gingr/src/webhook.rs) | Parse/verify separation, event/entity normalization, payload quarantine, acknowledgement status mapping. |
| Rustdoc/module paths | `gingr::config`, `gingr::endpoint`, `gingr::transport`, `gingr::response`, `gingr::dto`, `gingr::mapping`, `gingr::webhook` | Exact compiled contract once generated or published. |

## 7. Authoritative source system or human role

Ask “source of record for what?” before treating any Gingr value as authority.

| Fact or decision | Source of record | Human role when source is incomplete or sensitive |
| --- | --- | --- |
| Provider API documentation and endpoint shape | Gingr developer-guide articles captured in `docs/integrations/gingr/articles/` and summarized in `sdk-endpoint-catalog.md` | integration owner validates against future docs or test-tenant evidence |
| Tenant URL and API key | Runtime configuration supplied outside docs/fixtures | integration owner or operator; never put real keys in docs, fixtures, logs, screenshots, or comments |
| Provider owner, animal, reservation, reference, retail, report-card, timeclock, transaction, invoice, or subscription record | Gingr API response plus endpoint/source ref | staff or manager checks exceptions before business action |
| Provider event occurrence | HMAC-verified Gingr webhook envelope plus event/entity id | integration owner or workflow reviewer decides whether event is relevant and safely mapped |
| NVA customer/pet/product/reservation/workflow candidate | Explicit mapper output plus caller-owned source ref/provenance | workflow reviewer or manager accepts, corrects, or rejects candidate |
| Policy, safety, vaccine, temperament, incident, checkout, schedule, payment, or customer-outreach approval | NVA domain/app workflow contract plus human review outcome | front desk lead, manager, trained staff, or approved sender as applicable |
| Labor/result claim | Workflow outcome record in app/storage docs | reviewer who performed the work; regional ops interprets rollups |

## 8. Allowed actions

Gingr-boundary code and docs may safely describe or support these actions when a linked contract exists:

- read source-backed Gingr evidence through typed request builders;
- build redacted request diagnostics without printing secrets or sensitive lookup values;
- parse raw response/webhook payloads into quarantined provider DTOs;
- verify webhook signatures before exposing payload details to downstream mapping;
- map known owner, animal, and retail fields into explicit review candidates;
- preserve unknown fields and raw payload refs for audit without silently using them for policy;
- produce staff-facing packets, queues, summaries, or drafts for review;
- record mapping errors, review dispositions, source refs, outcome fields, and labor evidence.

Phrase examples carefully:

- Good: “The checkout workflow may read a Gingr reservation and transaction payload to create an exception candidate for staff review.”
- Good: “The webhook receiver may acknowledge a verified, durably accepted event, or reject an invalid signature permanently.”
- Bad: “NVA can check the pet out in Gingr.” The current boundary does not authorize provider writes.

## 9. Blocked actions and review gates

Default blocked actions unless a separate linked contract explicitly allows them:

- live Gingr provider writes, record hiding, or source-data deletion;
- customer/member/parent sends, marketing sends, or portal notifications;
- booking, check-in, checkout, schedule, room, yard, service, or package/session mutations;
- payment, refund, discount, deposit, invoice, estimate, subscription, or POS transaction movement;
- vaccine, medical, incident, temperament, group-play, feeding, medication, safety, or policy approvals;
- staff clock, staffing, payroll, personnel, or labor-schedule actions;
- use of customer passwords or authorization checks in normal data-ingestion flows;
- logging or publishing raw high-PII/payment/credential-bearing payloads;
- secret-dependent live external side effects from docs, tests, or agent workflows.

Use [draft](../../glossary-workflow-state-terms.md#draft), [review gate](../../glossary-workflow-state-terms.md#review-gate), and [blocked action](../../glossary-workflow-state-terms.md#blocked-action) language when this page feeds public/operator docs.

## 10. Safe-use evidence and outcome fields

Safe use requires enough evidence to reconstruct what was read, how it was transformed, and who reviewed the result. Look for:

- source system and provider endpoint/event name;
- provider record id or webhook entity id/type;
- redacted request capture or sanitized fixture reference;
- raw response/webhook payload reference, not pasted PII payload text;
- mapper name and mapping candidate type;
- mapping error kind when required provider fields are missing or invalid;
- review gate name, reviewer role, disposition, and correction/suppression reason;
- outcome record, audit event, or labor-minute field when claiming result value.

Tests and contracts that prove safety today include:

| Safety property | Evidence to check |
| --- | --- |
| Secrets do not leak through config/request diagnostics | `config` redaction tests and `transport::RedactedRequest` tests. |
| Endpoint builders encode documented provider shape | `endpoint::*` module tests for method, path, parameters, date range, pagination, and sensitive parameter names. |
| Sensitive owner lookups are marked | `owners_animals` tests for phone/email sensitive lookup fields. |
| Mapping refuses missing required provider fields | `mapping::{customer, pet, retail}` tests and errors for `ProviderField`. |
| Raw payloads stay quarantined | `response` DTO/envelope contracts and unknown-field preservation. |
| Webhook payloads require signature verification before workflow use | `webhook` tests for parse/verify, HMAC, malformed hex, missing fields, and ack status mapping. |
| Read-only/source inventory excludes side-effect endpoints | `sdk-endpoint-catalog.md` excludes `quick_checkin` and `receive_call`; `endpoint::catalog::exported_read_endpoint_names` and `semantic_mapping_gaps` show modeled reads/gaps. |

Do not claim “Gingr mapping is safe” generically. Name the specific mapper, endpoint, fixture, test, or contract that proves the narrow claim.

## 11. Examples and non-examples

| Type | Item | Why |
| --- | --- | --- |
| Example | `mapping::customer::ContactCandidate` from a Gingr `OwnerRecord` | It tells reviewers that NVA assembled contact evidence from provider fields, but it remains a candidate and source-backed draft input. |
| Example | `webhook::Verified` after `Envelope::verify` succeeds | It proves the envelope passed the HMAC gate before event/entity mapping; it still does not authorize a business action by itself. |
| Example | `transport::RedactedRequest` | It lets engineers/debuggers see method/path/non-sensitive parameters without exposing `key`, password, phone, email, or other marked sensitive fields. |
| Non-example | `endpoint::OwnerId` as a standalone customer entity | It is a provider identifier; document it inside the Gingr boundary or source/provenance story unless reconciled into a domain identity. |
| Non-example | Raw `response::provider::Payload` or unknown fields | They are audit/debug evidence and future mapping material, not policy or workflow facts. |
| Non-example | `dto::grooming::provider_surface()` or `dto::training::provider_surface()` as a mapped service contract | These explicitly mark areas where endpoints exist but semantic DTO mapping is intentionally absent. |
| Non-example | Gingr `email_sent` event as permission to email a customer | It reports provider email activity; NVA outreach still needs its own workflow approval and suppression rules. |

## 12. Glossary cross-links

Use these links when turning this family entry into public/operator docs:

- [Gingr](../../glossary-source-data-terms.md#domainsourcesystemgingr-gingr), [source of record](../../glossary-source-data-terms.md#source-of-record), [provider record](../../glossary-source-data-terms.md#provider-record), and [provenance/source ref](../../glossary-source-data-terms.md#domainsourceprovenance-and-domainsourcerecordref-as-data-evidence) for evidence language.
- [DTO](../../glossary-architecture-terms.md#dto), [adapter](../../glossary-architecture-terms.md#adapter), and [integration](../../glossary-architecture-terms.md#integration-integrationsgingr) for code-boundary language.
- [Draft](../../glossary-workflow-state-terms.md#draft), [review gate](../../glossary-workflow-state-terms.md#review-gate), [blocked action](../../glossary-workflow-state-terms.md#blocked-action), [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet), and [outcome capture](../../glossary-workflow-state-terms.md#outcome-capture) for safe workflow language.

## 13. Writer checklist for future Gingr atlas pages

Before splitting any row from this family page into its own atlas page, verify:

- the page says what Gingr says separately from what NVA derives;
- DTO names are translated into non-coder terms before source/module names appear;
- mapping code is described as transformation into candidates, not provider truth becoming domain truth automatically;
- allowed actions are read, validate, map, draft, rank, summarize, and record evidence only;
- blocked actions include provider writes, customer sends, schedule/check-in/out mutations, payment movement, safety approvals, and live secret-dependent side effects;
- secrets/config are described only as redacted runtime inputs, never copied into docs or fixtures;
- tests/contracts are named narrowly enough that a reviewer can find the proof.
